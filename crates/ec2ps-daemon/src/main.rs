//! ec2passd background daemon service.
//!
//! Listens for length-delimited JSON-RPC / streaming frames over TCP (`127.0.0.1:50051` by default)
//! and orchestrates the full Amazon Linux 2023 VM lifecycle:
//! - ImageVault CDN resolution, caching, and CoW QCOW2 overlay creation
//! - Pure-Rust `cidata` ISO9660 (`seed.iso`) cloud-init injection
//! - QEMU / QMP process orchestration
//! - SSH command execution, file transfer, live host-to-guest directory mounts, snapshots, and clones.

use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::Parser;
use ec2ps_core::catalog::ImageVault;
use ec2ps_core::cloud_init::{generate_al2023_seed_iso, validate_cloud_init_yaml, CloudInitConfig};
use ec2ps_core::default_data_dir;
use ec2ps_core::driver::VirtualMachineDriver;
use ec2ps_core::petname::generate_petname;
use ec2ps_core::ssh::{copy_dir_recursive, SshManager};
use ec2ps_core::state::{InstanceRecord, StateStore};
use ec2ps_protocol::{
    CommandAlias, InstanceDetails, InstanceState, MountSpec, NetworkInterfaceInfo, RpcRequest,
    RpcResponse, RpcStreamFrame,
};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(
    name = "ec2passd",
    about = "InhausLab ec2pass Daemon - Virtual Machine Daemon for Amazon Linux 2023"
)]
struct DaemonArgs {
    /// Address to bind the daemon RPC server (Windows named pipe or TCP host:port)
    #[arg(long, env = "EC2PASS_DAEMON_ADDR")]
    address: Option<String>,

    /// Data storage directory for images, instances, keys, and state
    #[arg(long, env = "EC2PASS_HOME")]
    data_dir: Option<PathBuf>,
}

struct DaemonContext {
    data_dir: PathBuf,
    store: Mutex<StateStore>,
    vault: ImageVault,
    ssh: SshManager,
    driver: Box<dyn VirtualMachineDriver>,
    shutdown_cancellers: Mutex<HashMap<String, tokio::sync::oneshot::Sender<()>>>,
    auth_token: String,
}

#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    unsafe {
        let res = libc::kill(pid as i32, 0);
        res == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
    }
}

#[cfg(windows)]
fn is_process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    let out = std::process::Command::new("cmd")
        .args(["/c", &format!("tasklist /FI \"PID eq {}\" /NH", pid)])
        .output();
    if let Ok(o) = out {
        let stdout = String::from_utf8_lossy(&o.stdout);
        stdout.contains(&pid.to_string())
    } else {
        false
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = DaemonArgs::parse();
    let mut data_dir = args.data_dir.unwrap_or_else(default_data_dir);
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        tracing::warn!(
            "Failed to create data dir at {:?}: {}. Falling back to .ec2ps-data",
            data_dir,
            e
        );
        data_dir = PathBuf::from(".ec2ps-data");
        std::fs::create_dir_all(&data_dir)?;
    }

    let mut store = StateStore::load_or_create(&data_dir)?;
    for rec in store.data.instances.values_mut() {
        if rec.details.state == InstanceState::Running {
            let is_alive = rec.pid.is_some_and(is_process_alive);
            if !is_alive {
                rec.details.state = InstanceState::Stopped;
                rec.pid = None;
            }
        }
    }
    let _ = store.save();
    let vault = ImageVault::new(data_dir.join("cache").join("images"))?;
    let ssh = SshManager::new(data_dir.join("ssh-keys"))?;
    let preferred_driver = store.data.settings.get("local.driver").map(|s| s.as_str());
    let driver = match ec2ps_core::driver::create_default_driver(preferred_driver) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            tracing::error!("Mandatory hypervisor requirement not satisfied: {}", e);
            std::process::exit(1);
        }
    };

    let bind_addr = args
        .address
        .unwrap_or_else(ec2ps_protocol::default_daemon_address);

    tracing::info!(
        "ec2passd (InhausLab ec2pass Daemon v{}) listening on {} [driver: {}, data_dir: {:?}]",
        env!("CARGO_PKG_VERSION"),
        bind_addr,
        driver.driver_name(),
        data_dir
    );

    let auth_token_path = data_dir.join("auth_token");
    let auth_token = if auth_token_path.exists() {
        std::fs::read_to_string(&auth_token_path)
            .unwrap_or_default()
            .trim()
            .to_string()
    } else {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let token = format!("{:016x}{:08x}", now, std::process::id());
        let _ = std::fs::write(&auth_token_path, &token);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(mut perms) = std::fs::metadata(&auth_token_path).map(|m| m.permissions()) {
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(&auth_token_path, perms);
            }
        }
        token
    };

    let ctx = Arc::new(DaemonContext {
        data_dir,
        store: Mutex::new(store),
        vault,
        ssh,
        driver,
        shutdown_cancellers: Mutex::new(HashMap::new()),
        auth_token,
    });

    if bind_addr.starts_with(r"\\.\pipe\") || bind_addr.starts_with("npipe:") {
        #[cfg(windows)]
        {
            use tokio::net::windows::named_pipe::ServerOptions;
            let pipe_name = if bind_addr.starts_with("npipe:") {
                bind_addr.trim_start_matches("npipe:")
            } else {
                &bind_addr
            };

            let mut server = ServerOptions::new()
                .first_pipe_instance(true)
                .create(pipe_name)
                .with_context(|| format!("Failed to create named pipe {}", pipe_name))?;

            tracing::info!(
                "ec2passd daemon is ready to accept client connections via Named Pipe {}",
                pipe_name
            );

            loop {
                tokio::select! {
                    connect_res = server.connect() => {
                        match connect_res {
                            Ok(()) => {
                                let connected_client = server;
                                server = ServerOptions::new().create(pipe_name)?;
                                let ctx_clone = Arc::clone(&ctx);
                                tokio::spawn(async move {
                                    if let Err(err) = handle_client(connected_client, ctx_clone).await {
                                        tracing::warn!("Named pipe client session error: {:#}", err);
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::warn!("Named pipe connect error: {:#}", e);
                            }
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        tracing::info!("Received shutdown signal (Ctrl+C). Saving state and exiting gracefully...");
                        let store = ctx.store.lock().await;
                        let _ = store.save();
                        break;
                    }
                }
            }
            return Ok(());
        }
        #[cfg(not(windows))]
        bail!("Named pipes are only supported on Windows");
    }

    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("Failed to bind daemon socket on {}", bind_addr))?;

    tracing::info!(
        "ec2passd daemon is ready to accept client connections via TCP {}",
        bind_addr
    );

    loop {
        tokio::select! {
            accept_res = listener.accept() => {
                match accept_res {
                    Ok((socket, _peer)) => {
                        let ctx_clone = Arc::clone(&ctx);
                        tokio::spawn(async move {
                            if let Err(err) = handle_client(socket, ctx_clone).await {
                                tracing::warn!("Client session error: {:#}", err);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::warn!("Listener accept error: {:#}", e);
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Received shutdown signal (Ctrl+C). Saving state and exiting gracefully...");
                let store = ctx.store.lock().await;
                let _ = store.save();
                break;
            }
        }
    }

    Ok(())
}

async fn send_frame<W>(writer: &mut W, frame: &RpcStreamFrame) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let mut line = serde_json::to_string(frame)?;
    line.push('\n');
    writer.write_all(line.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

async fn send_progress<W>(
    writer: &mut W,
    phase: &str,
    message: &str,
    percent: Option<u8>,
) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    send_frame(
        writer,
        &RpcStreamFrame::Progress {
            phase: phase.to_string(),
            message: message.to_string(),
            percent,
        },
    )
    .await
}

async fn handle_client<S>(socket: S, ctx: Arc<DaemonContext>) -> Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (reader, mut writer) = tokio::io::split(socket);
    let mut lines = BufReader::new(reader).lines();
    let mut session_authenticated = false;

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let req: RpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let _ = send_frame(
                    &mut writer,
                    &RpcStreamFrame::Response(Box::new(RpcResponse::Error(format!(
                        "Malformed RPC request: {}",
                        e
                    )))),
                )
                .await;
                continue;
            }
        };

        // Check if authentication is enabled
        let auth_required = {
            let store = ctx.store.lock().await;
            store
                .data
                .settings
                .get("local.passphrase")
                .map(|v| v == "true")
                .unwrap_or(false)
        };

        if auth_required && !session_authenticated {
            match &req {
                RpcRequest::Ping | RpcRequest::Version => {}
                RpcRequest::Authenticate { passphrase } => {
                    let store = ctx.store.lock().await;
                    let valid = if let Some(secret) = store.data.settings.get("local._passphrase_secret") {
                        !secret.is_empty() && (secret == passphrase || &ctx.auth_token == passphrase)
                    } else {
                        &ctx.auth_token == passphrase
                    };
                    if valid {
                        session_authenticated = true;
                        send_frame(
                            &mut writer,
                            &RpcStreamFrame::Response(Box::new(RpcResponse::OkMessage(
                                "Client authenticated successfully with ec2passd.".to_string(),
                            ))),
                        )
                        .await?;
                        continue;
                    } else {
                        send_frame(
                            &mut writer,
                            &RpcStreamFrame::Response(Box::new(RpcResponse::Error(
                                "Passphrase does not match local.passphrase".to_string(),
                            ))),
                        )
                        .await?;
                        continue;
                    }
                }
                _ => {
                    send_frame(
                        &mut writer,
                        &RpcStreamFrame::Response(Box::new(RpcResponse::Error(
                            "Unauthorized: authentication required. Run 'ec2pass authenticate' with the passphrase.".to_string(),
                        ))),
                    )
                    .await?;
                    continue;
                }
            }
        }

        if let RpcRequest::Authenticate { ref passphrase } = req {
            let store = ctx.store.lock().await;
            if let Some(secret) = store.data.settings.get("local._passphrase_secret") {
                if !secret.is_empty() && secret != passphrase && &ctx.auth_token != passphrase {
                    send_frame(
                        &mut writer,
                        &RpcStreamFrame::Response(Box::new(RpcResponse::Error(
                            "Passphrase does not match local.passphrase".to_string(),
                        ))),
                    )
                    .await?;
                    continue;
                }
            }
            session_authenticated = true;
            send_frame(
                &mut writer,
                &RpcStreamFrame::Response(Box::new(RpcResponse::OkMessage(
                    "Client authenticated successfully with ec2passd.".to_string(),
                ))),
            )
            .await?;
            continue;
        }

        let resp = match dispatch_request(req, &ctx, &mut writer).await {
            Ok(r) => r,
            Err(e) => RpcResponse::Error(format!("{:#}", e)),
        };

        send_frame(&mut writer, &RpcStreamFrame::Response(Box::new(resp))).await?;
    }
    Ok(())
}

async fn dispatch_request<W>(
    req: RpcRequest,
    ctx: &Arc<DaemonContext>,
    writer: &mut W,
) -> Result<RpcResponse>
where
    W: AsyncWriteExt + Unpin,
{
    match req {
        RpcRequest::Ping => Ok(RpcResponse::Pong),

        RpcRequest::Version => Ok(RpcResponse::Version {
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            daemon_version: env!("CARGO_PKG_VERSION").to_string(),
            driver: ctx.driver.driver_name().to_string(),
            default_image: "Amazon Linux 2023.6 LTS (al2023)".to_string(),
        }),

        RpcRequest::Authenticate { passphrase } => {
            let store = ctx.store.lock().await;
            if let Some(secret) = store.data.settings.get("local._passphrase_secret") {
                if !secret.is_empty() && secret != &passphrase && ctx.auth_token != passphrase {
                    bail!("Passphrase does not match local.passphrase");
                }
            }
            Ok(RpcResponse::OkMessage(
                "Client authenticated successfully with ec2passd.".to_string(),
            ))
        }

        RpcRequest::Find {
            search_string,
            show_unsupported,
            show_versions,
            force_update,
        } => {
            let mut entries = ctx
                .vault
                .fetch_catalog_extended(show_versions, force_update)
                .await;
            if !show_unsupported && search_string.is_none() {
                entries.retain(|e| !e.description.to_lowercase().contains("legacy"));
            }
            if let Some(q) = search_string {
                let q_low = q.to_lowercase();
                entries.retain(|e| {
                    e.release.to_lowercase().contains(&q_low)
                        || e.version.to_lowercase().contains(&q_low)
                        || e.description.to_lowercase().contains(&q_low)
                        || e.aliases.iter().any(|a| a.to_lowercase().contains(&q_low))
                });
            }
            Ok(RpcResponse::CatalogList(entries))
        }

        RpcRequest::Launch {
            image,
            image_version,
            name,
            cpus,
            memory_bytes,
            disk_bytes,
            cloud_init_yaml,
            networks,
            mounts,
            timeout_secs: req_timeout,
        } => {
            if let Some(ref yaml) = cloud_init_yaml {
                validate_cloud_init_yaml(yaml)?;
            }

            // 1. Determine instance name
            let instance_name = {
                let store = ctx.store.lock().await;
                let existing: Vec<String> = store.data.instances.keys().cloned().collect();
                match name {
                    Some(n) => {
                        if let Some(existing_rec) = store.data.instances.get(&n) {
                            if existing_rec.details.state == InstanceState::Deleted {
                                bail!(
                                    "Instance '{}' is deleted. Use 'recover' to restore it or 'purge' to permanently remove it.",
                                    n
                                );
                            }
                            bail!("Instance '{}' already exists.", n);
                        }
                        n
                    }
                    None => generate_petname(&existing),
                }
            };

            // 2. Resolve Amazon Linux 2023 image from ImageVault
            let catalog_entry = ctx
                .vault
                .resolve_image_with_version(image.as_deref(), image_version.as_deref())
                .await?;
            send_progress(
                writer,
                "Retrieving image",
                &format!(
                    "Retrieving image: {} ({})",
                    catalog_entry.release, catalog_entry.version
                ),
                None,
            )
            .await?;

            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(String, Option<u8>)>();
            let vault = ctx.vault.clone();
            let entry_clone = catalog_entry.clone();

            let download_task = async move {
                vault
                    .ensure_base_image(&entry_clone, move |msg, pct| {
                        let _ = tx.send((msg.to_string(), pct));
                    })
                    .await
            };
            tokio::pin!(download_task);

            let base_image_path = loop {
                tokio::select! {
                    Some((msg, pct)) = rx.recv() => {
                        send_progress(writer, "Retrieving image", &msg, pct).await?;
                    }
                    res = &mut download_task => {
                        while let Ok((msg, pct)) = rx.try_recv() {
                            send_progress(writer, "Retrieving image", &msg, pct).await?;
                        }
                        break res?;
                    }
                }
            };

            // 3. Prepare instance directory, CoW qcow2 overlay disk, and pure-Rust seed.iso (cidata)
            let instance_dir = ctx.data_dir.join("instances").join(&instance_name);
            std::fs::create_dir_all(&instance_dir)?;
            let disk_path = instance_dir.join("al2023-root.qcow2");
            let seed_iso_path = instance_dir.join("seed.iso");

            send_progress(
                writer,
                "Preparing",
                &format!("Creating QCOW2 overlay disk for {}...", instance_name),
                Some(45),
            )
            .await?;
            ctx.driver
                .create_overlay_disk(&base_image_path, &disk_path, disk_bytes)
                .await?;

            send_progress(
                writer,
                "Cloud-Init",
                &format!(
                    "Generating NoCloud cidata seed.iso for Amazon Linux 2023 ({})...",
                    instance_name
                ),
                Some(65),
            )
            .await?;

            let pub_key = ctx.ssh.public_key_string()?;
            generate_al2023_seed_iso(
                &seed_iso_path,
                &CloudInitConfig {
                    instance_id: &format!("i-al2023-{}", instance_name),
                    hostname: &instance_name,
                    default_user: "ec2-user",
                    ssh_public_key: &pub_key,
                    user_data_yaml: cloud_init_yaml.as_deref(),
                },
            )?;

            // 4. Allocate ports, IP, MAC, and build InstanceRecord
            let (ssh_port, qmp_port, guest_ip, mac_addr) = {
                let mut store = ctx.store.lock().await;
                let (s, q) = store.allocate_ports();
                let ip = store.allocate_guest_ip();
                let mac = store.allocate_mac();
                (s, q, ip, mac)
            };

            let mut mount_map = BTreeMap::new();
            for (src, dst) in mounts {
                mount_map.insert(
                    dst.clone(),
                    MountSpec {
                        source_path: src.clone(),
                        target_path: dst.clone(),
                        uid_map: vec![(1000, 1000)],
                        gid_map: vec![(1000, 1000)],
                        mount_type: "native".to_string(),
                    },
                );
            }

            let details = InstanceDetails {
                name: instance_name.clone(),
                state: InstanceState::Starting,
                ipv4: vec![guest_ip],
                release: catalog_entry.release.clone(),
                image_hash: catalog_entry.sha256.clone(),
                image_release: catalog_entry.version.clone(),
                cpus,
                load: vec![0.0, 0.0, 0.0],
                memory_usage_bytes: 0,
                memory_total_bytes: memory_bytes,
                disk_usage_bytes: std::fs::metadata(&disk_path).map(|m| m.len()).unwrap_or(0),
                disk_total_bytes: disk_bytes,
                mounts: mount_map.clone(),
                ssh_port,
                ssh_user: "ec2-user".to_string(),
                ssh_key_path: ctx.ssh.private_key_path().to_string_lossy().to_string(),
                snapshots: Vec::new(),
                created_at: Utc::now(),
            };

            let mut record = InstanceRecord {
                details,
                instance_dir: instance_dir.clone(),
                disk_path,
                seed_iso_path,
                base_image_path,
                qmp_port,
                pid: None,
                mac_address: mac_addr,
                cloud_init_yaml,
                networks,
            };

            // 5. Boot VM with QEMU Driver
            send_progress(
                writer,
                "Launching",
                &format!("Launching {}...", instance_name),
                Some(80),
            )
            .await?;
            let pid = ctx.driver.start_vm(&record).await?;
            record.pid = Some(pid);

            send_progress(
                writer,
                "Waiting",
                &format!(
                    "Waiting for {} to boot and cloud-init to complete...",
                    instance_name
                ),
                Some(85),
            )
            .await?;

            let start_wait = std::time::Instant::now();
            let timeout_secs = if req_timeout == 0 { 300 } else { req_timeout };
            let mut is_ready = false;

            while start_wait.elapsed() < std::time::Duration::from_secs(timeout_secs) {
                let elapsed = start_wait.elapsed().as_secs();
                let pct = (85 + (elapsed as f64 / timeout_secs as f64 * 14.0) as u8).min(99);

                // 1. Check if SSH TCP port is open (1.5s timeout)
                let addr = format!("127.0.0.1:{}", ssh_port);
                let tcp_ok = matches!(
                    tokio::time::timeout(
                        std::time::Duration::from_millis(1500),
                        tokio::net::TcpStream::connect(&addr),
                    )
                    .await,
                    Ok(Ok(_))
                );

                if tcp_ok {
                    send_progress(
                        writer,
                        "Waiting",
                        &format!(
                            "SSH port open. Verifying cloud-init status ({}/{}s)...",
                            elapsed, timeout_secs
                        ),
                        Some(pct),
                    )
                    .await?;

                    if let Ok((stdout, _stderr, code)) = ctx
                        .ssh
                        .exec_command(
                            &instance_name,
                            &instance_dir,
                            ssh_port,
                            "ec2-user",
                            &["cloud-init status --wait || true".to_string()],
                            None,
                        )
                        .await
                    {
                        if code == 0 || stdout.contains("done") || stdout.contains("status:") {
                            is_ready = true;
                            break;
                        }
                    }
                } else {
                    send_progress(
                        writer,
                        "Waiting",
                        &format!(
                            "Waiting for instance to boot ({}/{}s)...",
                            elapsed, timeout_secs
                        ),
                        Some(pct),
                    )
                    .await?;
                }

                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }

            if !is_ready {
                let _ = ctx.driver.stop_vm(&record, true).await;
                record.details.state = InstanceState::Stopped;
                let mut store = ctx.store.lock().await;
                store.data.instances.insert(instance_name.clone(), record);
                store.save()?;
                bail!(
                    "Timed out waiting for instance '{}' to initialize (SSH port {} did not respond after {}s).",
                    instance_name,
                    ssh_port,
                    timeout_secs
                );
            }

            record.details.state = InstanceState::Running;
            let launched_details = record.details.clone();

            for (idx, (tgt, m)) in mount_map.iter().enumerate() {
                let tag = format!("ec2ps_mount_{}", idx);
                let _ = ctx
                    .ssh
                    .sync_mount(&instance_dir, ssh_port, "ec2-user", &m.source_path, tgt, Some(&tag))
                    .await;
            }

            {
                let mut store = ctx.store.lock().await;
                store.data.instances.insert(instance_name, record);
                store.save()?;
            }

            send_progress(
                writer,
                "Ready",
                &format!("Instance {} is ready and running.", launched_details.name),
                Some(100),
            )
            .await?;

            Ok(RpcResponse::Launched(launched_details))
        }

        RpcRequest::Start { names, all } => {
            let mut store = ctx.store.lock().await;
            let targets = resolve_target_names(&store, &names, all, "primary")?;
            let mut started = Vec::new();

            // If `ec2pass start` was called with default "primary" and it doesn't exist, error out cleanly
            for name in targets {
                let rec = store
                    .data
                    .instances
                    .get_mut(&name)
                    .with_context(|| format!("Instance '{}' does not exist", name))?;
                if rec.details.state == InstanceState::Deleted {
                    bail!(
                        "Cannot start deleted instance '{}'. Recover it first.",
                        name
                    );
                }
                let is_alive = rec.pid.is_some_and(is_process_alive);
                if rec.details.state == InstanceState::Suspended {
                    ctx.driver.resume_vm(rec).await?;
                    let _ = ctx
                        .ssh
                        .wait_until_ready(
                            &name,
                            &rec.instance_dir,
                            rec.details.ssh_port,
                            &rec.details.ssh_user,
                            15,
                        )
                        .await;
                } else if rec.details.state != InstanceState::Running || !is_alive {
                    let pid = ctx.driver.start_vm(rec).await?;
                    rec.pid = Some(pid);
                    let _ = ctx
                        .ssh
                        .wait_until_ready(
                            &name,
                            &rec.instance_dir,
                            rec.details.ssh_port,
                            &rec.details.ssh_user,
                            60,
                        )
                        .await;

                    // Automatically grow partition and filesystem if disk was resized while stopped
                    let grow_cmd = vec![
                        "sudo growpart /dev/vda 1 2>/dev/null || sudo growpart /dev/xvda 1 2>/dev/null || sudo growpart /dev/sda 1 2>/dev/null || true".to_string(),
                        "sudo xfs_growfs / 2>/dev/null || sudo resize2fs $(findmnt -n -o SOURCE / 2>/dev/null) 2>/dev/null || true".to_string(),
                    ];
                    let _ = ctx
                        .ssh
                        .exec_command(
                            &name,
                            &rec.instance_dir,
                            rec.details.ssh_port,
                            &rec.details.ssh_user,
                            &grow_cmd,
                            None,
                        )
                        .await;
                }
                rec.details.state = InstanceState::Running;
                for (idx, (tgt, m)) in rec.details.mounts.iter().enumerate() {
                    let tag = format!("ec2ps_mount_{}", idx);
                    let _ = ctx
                        .ssh
                        .sync_mount(&rec.instance_dir, rec.details.ssh_port, &rec.details.ssh_user, &m.source_path, tgt, Some(&tag))
                        .await;
                }
                started.push(rec.details.clone());
            }
            store.save()?;
            Ok(RpcResponse::InstanceList(started))
        }

        RpcRequest::Stop {
            names,
            all,
            force,
            delay_mins,
            cancel_shutdown,
        } => {
            let mut store = ctx.store.lock().await;
            let targets = resolve_target_names(&store, &names, all, "primary")?;
            let mut updated = Vec::new();

            for name in targets {
                let rec = store
                    .data
                    .instances
                    .get_mut(&name)
                    .with_context(|| format!("Instance '{}' does not exist", name))?;
                if cancel_shutdown {
                    if let Some(tx) = ctx.shutdown_cancellers.lock().await.remove(&name) {
                        let _ = tx.send(());
                    }
                    if rec.details.state == InstanceState::DelayedShutdown {
                        rec.details.state = InstanceState::Running;
                    }
                } else if let Some(mins) = delay_mins {
                    if mins > 0 {
                        // Cancel any previous pending timer
                        if let Some(tx) = ctx.shutdown_cancellers.lock().await.remove(&name) {
                            let _ = tx.send(());
                        }
                        let (tx, rx) = tokio::sync::oneshot::channel();
                        ctx.shutdown_cancellers.lock().await.insert(name.clone(), tx);
                        rec.details.state = InstanceState::DelayedShutdown;

                        let ctx_bg = Arc::clone(&ctx);
                        let name_bg = name.clone();
                        tokio::spawn(async move {
                            tokio::select! {
                                _ = tokio::time::sleep(std::time::Duration::from_secs(mins as u64 * 60)) => {
                                    ctx_bg.shutdown_cancellers.lock().await.remove(&name_bg);
                                    let mut st = ctx_bg.store.lock().await;
                                    if let Some(instance_rec) = st.data.instances.get_mut(&name_bg) {
                                        if instance_rec.details.state == InstanceState::DelayedShutdown {
                                            tracing::info!("Delayed shutdown timer fired for '{}'. Stopping VM...", name_bg);
                                            let _ = ctx_bg.driver.stop_vm(instance_rec, force).await;
                                            instance_rec.details.state = InstanceState::Stopped;
                                            instance_rec.pid = None;
                                            let _ = st.save();
                                        }
                                    }
                                }
                                _ = rx => {
                                    tracing::info!("Delayed shutdown cancelled for '{}'", name_bg);
                                }
                            }
                        });
                    } else {
                        ctx.driver.stop_vm(rec, force).await?;
                        rec.details.state = InstanceState::Stopped;
                        rec.pid = None;
                    }
                } else {
                    ctx.driver.stop_vm(rec, force).await?;
                    rec.details.state = InstanceState::Stopped;
                    rec.pid = None;
                }
                updated.push(rec.details.clone());
            }
            store.save()?;
            Ok(RpcResponse::InstanceList(updated))
        }

        RpcRequest::Restart { names, all } => {
            let mut store = ctx.store.lock().await;
            let targets = resolve_target_names(&store, &names, all, "primary")?;
            let mut restarted = Vec::new();

            for name in targets {
                let rec = store
                    .data
                    .instances
                    .get_mut(&name)
                    .with_context(|| format!("Instance '{}' does not exist", name))?;
                if rec.details.state != InstanceState::Running {
                    bail!("Instance '{}' is not running", name);
                }
                ctx.driver.stop_vm(rec, false).await?;
                let pid = ctx.driver.start_vm(rec).await?;
                rec.pid = Some(pid);
                let _ = ctx
                    .ssh
                    .wait_until_ready(
                        &name,
                        &rec.instance_dir,
                        rec.details.ssh_port,
                        &rec.details.ssh_user,
                        60,
                    )
                    .await;
                rec.details.state = InstanceState::Running;
                restarted.push(rec.details.clone());
            }
            store.save()?;
            Ok(RpcResponse::InstanceList(restarted))
        }

        RpcRequest::Suspend { names, all } => {
            let mut store = ctx.store.lock().await;
            let targets = resolve_target_names(&store, &names, all, "primary")?;
            let mut suspended = Vec::new();

            for name in targets {
                let rec = store
                    .data
                    .instances
                    .get_mut(&name)
                    .with_context(|| format!("Instance '{}' does not exist", name))?;
                if rec.details.state == InstanceState::Running {
                    ctx.driver.suspend_vm(rec).await?;
                    rec.details.state = InstanceState::Suspended;
                }
                suspended.push(rec.details.clone());
            }
            store.save()?;
            Ok(RpcResponse::InstanceList(suspended))
        }

        RpcRequest::Delete { names, all, purge } => {
            let mut store = ctx.store.lock().await;
            let targets = resolve_target_names(&store, &names, all, "primary")?;

            for name in targets {
                if let Some(rec) = store.data.instances.get_mut(&name) {
                    if rec.details.state == InstanceState::Running
                        || rec.details.state == InstanceState::Suspended
                    {
                        let _ = ctx.driver.stop_vm(rec, true).await;
                    }
                    rec.details.state = InstanceState::Deleted;
                    rec.pid = None;
                } else {
                    bail!("Instance '{}' does not exist", name);
                }
                if purge {
                    if let Some(removed) = store.data.instances.remove(&name) {
                        let _ = std::fs::remove_dir_all(&removed.instance_dir);
                    }
                }
            }
            store.save()?;
            Ok(RpcResponse::OkMessage("Deleted".to_string()))
        }

        RpcRequest::Recover { names, all } => {
            let mut store = ctx.store.lock().await;
            let targets = resolve_target_names(&store, &names, all, "primary")?;
            let mut recovered = Vec::new();

            for name in targets {
                let rec = store
                    .data
                    .instances
                    .get_mut(&name)
                    .with_context(|| format!("Instance '{}' does not exist", name))?;
                if rec.details.state == InstanceState::Deleted {
                    rec.details.state = InstanceState::Stopped;
                }
                recovered.push(rec.details.clone());
            }
            store.save()?;
            Ok(RpcResponse::InstanceList(recovered))
        }

        RpcRequest::Purge => {
            let mut store = ctx.store.lock().await;
            let deleted_names: Vec<String> = store
                .data
                .instances
                .iter()
                .filter(|(_, r)| r.details.state == InstanceState::Deleted)
                .map(|(k, _)| k.clone())
                .collect();

            for name in deleted_names {
                if let Some(removed) = store.data.instances.remove(&name) {
                    let _ = std::fs::remove_dir_all(&removed.instance_dir);
                }
            }
            store.save()?;
            Ok(RpcResponse::OkMessage(
                "Purged all deleted instances.".to_string(),
            ))
        }

        RpcRequest::List { snapshots: _ } => {
            let mut store = ctx.store.lock().await;
            let mut list = Vec::with_capacity(store.data.instances.len());
            for rec in store.data.instances.values_mut() {
                if rec.details.state == InstanceState::Running {
                    collect_instance_metrics(&ctx.ssh, rec).await;
                } else {
                    if let Ok(meta) = std::fs::metadata(&rec.disk_path) {
                        rec.details.disk_usage_bytes = meta.len();
                    }
                    rec.details.load = vec![0.0, 0.0, 0.0];
                    rec.details.memory_usage_bytes = 0;
                }
                list.push(rec.details.clone());
            }
            Ok(RpcResponse::InstanceList(list))
        }

        RpcRequest::Info {
            names,
            all,
            snapshots: _,
        } => {
            let mut store = ctx.store.lock().await;
            let targets = if names.is_empty() && !all {
                store.data.instances.keys().cloned().collect()
            } else {
                resolve_target_names(&store, &names, all, "primary")?
            };

            let mut list = Vec::with_capacity(targets.len());
            for name in targets {
                let rec = store
                    .data
                    .instances
                    .get_mut(&name)
                    .with_context(|| format!("Instance '{}' does not exist", name))?;
                collect_instance_metrics(&ctx.ssh, rec).await;
                list.push(rec.details.clone());
            }
            Ok(RpcResponse::InstanceList(list))
        }

        RpcRequest::Exec {
            name,
            command,
            working_dir,
        } => {
            let (instance_dir, ssh_port, ssh_user, state) = {
                let store = ctx.store.lock().await;
                let rec = store
                    .data
                    .instances
                    .get(&name)
                    .with_context(|| format!("Instance '{}' does not exist", name))?;
                (
                    rec.instance_dir.clone(),
                    rec.details.ssh_port,
                    rec.details.ssh_user.clone(),
                    rec.details.state,
                )
            };

            if state != InstanceState::Running {
                bail!(
                    "Instance '{}' is not running (current state: {})",
                    name,
                    state
                );
            }

            let (stdout, stderr, exit_code) = ctx
                .ssh
                .exec_command(
                    &name,
                    &instance_dir,
                    ssh_port,
                    &ssh_user,
                    &command,
                    working_dir.as_deref(),
                )
                .await?;

            Ok(RpcResponse::ExecResult {
                stdout,
                stderr,
                exit_code,
            })
        }

        RpcRequest::ShellInfo { name } => {
            let target_name = {
                let store = ctx.store.lock().await;
                let primary = store
                    .data
                    .settings
                    .get("client.primary-name")
                    .cloned()
                    .unwrap_or_else(|| "primary".to_string());
                name.unwrap_or(primary)
            };

            // Check if instance exists; if not and it's the primary instance, caller can auto-launch or we return error
            let store = ctx.store.lock().await;
            let rec = store
                .data
                .instances
                .get(&target_name)
                .with_context(|| format!("Instance '{}' does not exist", target_name))?;

            Ok(RpcResponse::ShellConnection {
                instance: rec.details.clone(),
                ssh_host: "127.0.0.1".to_string(),
                ssh_port: rec.details.ssh_port,
                ssh_user: rec.details.ssh_user.clone(),
                ssh_key_path: rec.details.ssh_key_path.clone(),
            })
        }

        RpcRequest::Mount {
            source_path,
            instance,
            target_path,
            uid_map,
            gid_map,
            mount_type,
        } => {
            let src = PathBuf::from(&source_path);
            if !src.exists() {
                bail!("Source path '{}' does not exist on host", source_path);
            }
            let abs_src = std::fs::canonicalize(&src)
                .unwrap_or(src)
                .to_string_lossy()
                .trim_start_matches(r"\\?\")
                .to_string();

            let (is_running, instance_dir, ssh_port, ssh_user, tag) = {
                let mut store = ctx.store.lock().await;
                if !store.data.instances.contains_key(&instance) {
                    bail!("No such instance: '{}'", instance);
                }

                store.add_mount(
                    &instance,
                    abs_src.clone(),
                    target_path.clone(),
                    uid_map,
                    gid_map,
                    mount_type,
                )?;
                let rec = &store.data.instances[&instance];
                let mount_idx = rec.details.mounts.len().saturating_sub(1);
                (
                    rec.details.state == InstanceState::Running,
                    rec.instance_dir.clone(),
                    rec.details.ssh_port,
                    rec.details.ssh_user.clone(),
                    format!("ec2ps_mount_{}", mount_idx),
                )
            };

            if is_running {
                ctx.ssh
                    .sync_mount(&instance_dir, ssh_port, &ssh_user, &abs_src, &target_path, Some(&tag))
                    .await?;
            }
            Ok(RpcResponse::OkMessage("Mounted".to_string()))
        }

        RpcRequest::Umount {
            instance,
            target_path,
        } => {
            let (is_running, instance_dir, ssh_port, ssh_user, targets_to_clean) = {
                let store = ctx.store.lock().await;
                let rec = store
                    .data
                    .instances
                    .get(&instance)
                    .with_context(|| format!("No such instance: '{}'", instance))?;
                let is_running = rec.details.state == InstanceState::Running;
                let targets = match &target_path {
                    Some(p) => vec![p.clone()],
                    None => rec.details.mounts.keys().cloned().collect(),
                };
                (
                    is_running,
                    rec.instance_dir.clone(),
                    rec.details.ssh_port,
                    rec.details.ssh_user.clone(),
                    targets,
                )
            };

            if is_running {
                for target in targets_to_clean {
                    let unmount_cmd = vec![
                        // If the target is an active mountpoint, unmount it cleanly
                        format!(
                            "if mountpoint -q '{0}' 2>/dev/null; then sudo umount -f '{0}' 2>/dev/null || sudo umount -l '{0}' 2>/dev/null || true; fi",
                            target
                        ),
                        // Only remove the mountpoint directory if it is completely empty; do not delete any files
                        format!("sudo rmdir '{0}' 2>/dev/null || true", target),
                    ];
                    let _ = ctx
                        .ssh
                        .exec_command(
                            &instance,
                            &instance_dir,
                            ssh_port,
                            &ssh_user,
                            &unmount_cmd,
                            None,
                        )
                        .await;
                }
            }

            let mut store = ctx.store.lock().await;
            store.remove_mount(&instance, target_path.as_deref())?;
            Ok(RpcResponse::OkMessage("Unmounted".to_string()))
        }

        RpcRequest::Transfer {
            sources,
            destination,
            recursive,
            parents,
        } => {
            let dst_endpoint = parse_endpoint(&destination);

            for src in &sources {
                let src_endpoint = parse_endpoint(src);

                match (src_endpoint, dst_endpoint) {
                    (Some((src_inst, src_path)), Some((dst_inst, dst_path))) => {
                        // Instance-to-Instance transfer via temporary host staging
                        let (src_dir, src_port, src_user, dst_dir, dst_port, dst_user) = {
                            let store = ctx.store.lock().await;
                            let s_rec = store
                                .data
                                .instances
                                .get(src_inst)
                                .with_context(|| format!("Source instance '{}' does not exist", src_inst))?;
                            let d_rec = store
                                .data
                                .instances
                                .get(dst_inst)
                                .with_context(|| format!("Destination instance '{}' does not exist", dst_inst))?;
                            (
                                s_rec.instance_dir.clone(),
                                s_rec.details.ssh_port,
                                s_rec.details.ssh_user.clone(),
                                d_rec.instance_dir.clone(),
                                d_rec.details.ssh_port,
                                d_rec.details.ssh_user.clone(),
                            )
                        };

                        let tmp_dir = std::env::temp_dir().join(format!(
                            "ec2ps_xfer_{}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_nanos()
                        ));
                        std::fs::create_dir_all(&tmp_dir)?;

                        let file_name = std::path::Path::new(src_path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("transfer.tmp");
                        let host_tmp_file = tmp_dir.join(file_name);

                        // 1. Transfer from source guest to host staging
                        let src_spec = format!("{}:{}", src_inst, src_path);
                        ctx.ssh
                            .transfer_path(
                                &src_dir,
                                src_port,
                                &src_user,
                                &src_spec,
                                &host_tmp_file.to_string_lossy(),
                                recursive,
                                parents,
                            )
                            .await?;

                        // 2. Transfer from host staging to destination guest
                        let dst_spec = format!("{}:{}", dst_inst, dst_path);
                        let res = ctx
                            .ssh
                            .transfer_path(
                                &dst_dir,
                                dst_port,
                                &dst_user,
                                &host_tmp_file.to_string_lossy(),
                                &dst_spec,
                                recursive,
                                parents,
                            )
                            .await;

                        let _ = std::fs::remove_dir_all(&tmp_dir);
                        res?;
                    }
                    (Some((src_inst, _)), None) => {
                        // Guest-to-Host transfer
                        let (dir, port, user) = {
                            let store = ctx.store.lock().await;
                            let rec = store
                                .data
                                .instances
                                .get(src_inst)
                                .with_context(|| format!("Source instance '{}' does not exist", src_inst))?;
                            (rec.instance_dir.clone(), rec.details.ssh_port, rec.details.ssh_user.clone())
                        };
                        ctx.ssh
                            .transfer_path(&dir, port, &user, src, &destination, recursive, parents)
                            .await?;
                    }
                    (None, Some((dst_inst, _))) => {
                        // Host-to-Guest transfer
                        let (dir, port, user) = {
                            let store = ctx.store.lock().await;
                            let rec = store
                                .data
                                .instances
                                .get(dst_inst)
                                .with_context(|| format!("Destination instance '{}' does not exist", dst_inst))?;
                            (rec.instance_dir.clone(), rec.details.ssh_port, rec.details.ssh_user.clone())
                        };
                        ctx.ssh
                            .transfer_path(&dir, port, &user, src, &destination, recursive, parents)
                            .await?;
                    }
                    (None, None) => {
                        bail!("Transfer requires at least one instance path in the form <instance>:<path>");
                    }
                }
            }
            Ok(RpcResponse::OkMessage("Transfer completed".to_string()))
        }

        RpcRequest::Snapshot {
            instance,
            snapshot_name,
            comment,
        } => {
            let mut store = ctx.store.lock().await;
            let (is_running, qmp_port, disk_path) = {
                let rec = store
                    .data
                    .instances
                    .get(&instance)
                    .with_context(|| format!("No such instance: '{}'", instance))?;
                (
                    rec.details.state == InstanceState::Running,
                    rec.qmp_port,
                    rec.disk_path.clone(),
                )
            };
            let snap = store.create_snapshot(&instance, snapshot_name, comment)?;
            if is_running {
                ctx.driver
                    .take_live_snapshot(qmp_port, &disk_path, &snap.name)
                    .await?;
            } else {
                ctx.driver
                    .take_disk_snapshot(&disk_path, &snap.name)
                    .await?;
            }
            Ok(RpcResponse::SnapshotCreated(snap))
        }

        RpcRequest::Restore {
            instance,
            snapshot_name,
            destructive,
        } => {
            let (is_running, qmp_port, disk_path, rec_clone) = {
                let store = ctx.store.lock().await;
                let rec = store
                    .data
                    .instances
                    .get(&instance)
                    .with_context(|| format!("No such instance: '{}'", instance))?;
                (
                    rec.details.state == InstanceState::Running,
                    rec.qmp_port,
                    rec.disk_path.clone(),
                    rec.clone(),
                )
            };
            let snap = {
                let mut store = ctx.store.lock().await;
                store.restore_snapshot(&instance, &snapshot_name, destructive)?
            };

            if is_running {
                let live_res = ctx
                    .driver
                    .restore_live_snapshot(qmp_port, &disk_path, &snap.name)
                    .await;
                if let Err(e) = live_res {
                    tracing::info!(
                        "Live QMP restore unavailable ({:?}); cleanly restarting instance to apply disk snapshot...",
                        e
                    );
                    ctx.driver.stop_vm(&rec_clone, true).await?;
                    ctx.driver
                        .restore_disk_snapshot(&disk_path, &snap.name)
                        .await?;
                    let pid = ctx.driver.start_vm(&rec_clone).await?;
                    {
                        let mut store = ctx.store.lock().await;
                        if let Some(r) = store.data.instances.get_mut(&instance) {
                            r.pid = Some(pid);
                            r.details.state = InstanceState::Running;
                        }
                        store.save()?;
                    }
                    let _ = ctx
                        .ssh
                        .wait_until_ready(
                            &instance,
                            &rec_clone.instance_dir,
                            rec_clone.details.ssh_port,
                            &rec_clone.details.ssh_user,
                            60,
                        )
                        .await;
                }
            } else {
                ctx.driver
                    .restore_disk_snapshot(&disk_path, &snap.name)
                    .await?;
            }
            Ok(RpcResponse::SnapshotCreated(snap))
        }

        RpcRequest::Clone {
            source_name,
            destination_name,
        } => {
            let mut store = ctx.store.lock().await;
            let src_rec = store
                .data
                .instances
                .get(&source_name)
                .cloned()
                .with_context(|| format!("Source instance '{}' does not exist", source_name))?;

            if src_rec.details.state != InstanceState::Stopped {
                bail!(
                    "Source instance '{}' must be Stopped before cloning (currently {}).",
                    source_name,
                    src_rec.details.state
                );
            }

            let dst_name = destination_name
                .unwrap_or_else(|| format!("{}-clone{}", source_name, store.data.instances.len()));
            if store.data.instances.contains_key(&dst_name) {
                bail!("Destination instance '{}' already exists", dst_name);
            }

            let dst_dir = ctx.data_dir.join("instances").join(&dst_name);
            copy_dir_recursive(&src_rec.instance_dir, &dst_dir)?;

            // Clean up stale runtime logs/sockets in clone
            let _ = std::fs::remove_file(dst_dir.join("console.log"));
            let _ = std::fs::remove_file(dst_dir.join("qmp.sock"));

            let (ssh_port, qmp_port) = store.allocate_ports();
            let guest_ip = store.allocate_guest_ip();
            let mac = store.allocate_mac();

            let mut dst_rec = src_rec.clone();
            dst_rec.details.name = dst_name.clone();
            dst_rec.details.state = InstanceState::Stopped;
            dst_rec.pid = None;
            dst_rec.details.ipv4 = vec![guest_ip];
            dst_rec.details.ssh_port = ssh_port;
            dst_rec.details.snapshots.clear();
            dst_rec.details.created_at = Utc::now();
            dst_rec.instance_dir = dst_dir.clone();
            dst_rec.disk_path = dst_dir.join("al2023-root.qcow2");
            dst_rec.seed_iso_path = dst_dir.join("seed.iso");
            dst_rec.qmp_port = qmp_port;
            dst_rec.mac_address = mac;

            // Regenerate cloud-init seed.iso with new instance-id and hostname to prevent cloud-init conflict
            let pub_key = ctx.ssh.public_key_string()?;
            generate_al2023_seed_iso(
                &dst_rec.seed_iso_path,
                &CloudInitConfig {
                    instance_id: &format!("i-al2023-{}", dst_name),
                    hostname: &dst_name,
                    default_user: "ec2-user",
                    ssh_public_key: &pub_key,
                    user_data_yaml: dst_rec.cloud_init_yaml.as_deref(),
                },
            )?;

            let details = dst_rec.details.clone();
            store.data.instances.insert(dst_name, dst_rec);
            store.save()?;
            Ok(RpcResponse::Launched(details))
        }

        RpcRequest::Get { key, keys_only } => {
            let store = ctx.store.lock().await;
            let all_settings = store.get_all_settings();

            if keys_only {
                let mut only_keys = BTreeMap::new();
                for k in all_settings.keys() {
                    if !k.starts_with("local._") {
                        only_keys.insert(k.clone(), String::new());
                    }
                }
                return Ok(RpcResponse::SettingsMap(only_keys));
            }

            if let Some(k) = key {
                if let Some(val) = all_settings.get(&k) {
                    let mut single = BTreeMap::new();
                    single.insert(k, val.clone());
                    return Ok(RpcResponse::SettingsMap(single));
                }
                // Prefix match (e.g., `ec2pass get local`)
                let prefix = format!("{}.", k);
                let matched: BTreeMap<String, String> = all_settings
                    .into_iter()
                    .filter(|(key_name, _)| {
                        key_name.starts_with(&prefix) && !key_name.starts_with("local._")
                    })
                    .collect();
                if matched.is_empty() {
                    bail!("Unrecognized settings key: '{}'", k);
                }
                return Ok(RpcResponse::SettingsMap(matched));
            }

            let filtered: BTreeMap<String, String> = all_settings
                .into_iter()
                .filter(|(k, _)| !k.starts_with("local._"))
                .collect();
            Ok(RpcResponse::SettingsMap(filtered))
        }

        RpcRequest::Set { key, value } => {
            let parts: Vec<&str> = key.split('.').collect();
            if parts.len() == 3 && parts[0] == "local" && parts[2] == "disk" {
                let inst_name = parts[1];
                let (disk_path, state, current_bytes) = {
                    let store = ctx.store.lock().await;
                    let rec = store
                        .data
                        .instances
                        .get(inst_name)
                        .with_context(|| format!("No such instance: '{}'", inst_name))?;
                    (
                        rec.disk_path.clone(),
                        rec.details.state,
                        rec.details.disk_total_bytes,
                    )
                };

                if state != InstanceState::Stopped {
                    bail!(
                        "Cannot resize instance '{}' while it is {}. Please stop the instance first.",
                        inst_name,
                        state
                    );
                }

                let new_bytes = ec2ps_protocol::parse_byte_size(&value)
                    .context("Invalid disk size format")?;
                if new_bytes < current_bytes {
                    bail!(
                        "Disk size can only be increased (current: {}, requested: {})",
                        ec2ps_protocol::format_bytes(current_bytes),
                        ec2ps_protocol::format_bytes(new_bytes)
                    );
                }

                ctx.driver.resize_disk(&disk_path, new_bytes).await?;
            }

            let mut store = ctx.store.lock().await;
            store.set_setting(&key, &value)?;
            Ok(RpcResponse::OkMessage(format!("{}={}", key, value)))
        }

        RpcRequest::Alias {
            instance,
            command,
            alias_name,
            working_directory,
        } => {
            let mut store = ctx.store.lock().await;
            if !store.data.instances.contains_key(&instance) {
                bail!("Instance '{}' does not exist", instance);
            }
            let name = alias_name.unwrap_or_else(|| command.clone());
            store.data.aliases.insert(
                name.clone(),
                CommandAlias {
                    alias_name: name,
                    instance,
                    command,
                    working_directory,
                },
            );
            store.save()?;
            Ok(RpcResponse::OkMessage("Alias created".to_string()))
        }

        RpcRequest::Unalias { alias_names, all } => {
            let mut store = ctx.store.lock().await;
            if all {
                store.data.aliases.clear();
            } else {
                for a in alias_names {
                    if store.data.aliases.remove(&a).is_none() {
                        bail!("Alias '{}' does not exist", a);
                    }
                }
            }
            store.save()?;
            Ok(RpcResponse::OkMessage("Alias removed".to_string()))
        }

        RpcRequest::Aliases => {
            let store = ctx.store.lock().await;
            let list: Vec<CommandAlias> = store.data.aliases.values().cloned().collect();
            Ok(RpcResponse::AliasList(list))
        }

        RpcRequest::Networks => {
            let nets = detect_host_networks().await;
            Ok(RpcResponse::NetworkList(nets))
        }
    }
}

async fn collect_instance_metrics(
    ssh: &ec2ps_core::ssh::SshManager,
    rec: &mut ec2ps_core::state::InstanceRecord,
) {
    if let Ok(meta) = std::fs::metadata(&rec.disk_path) {
        rec.details.disk_usage_bytes = meta.len();
    }

    if rec.details.state != InstanceState::Running {
        rec.details.load = vec![0.0, 0.0, 0.0];
        rec.details.memory_usage_bytes = 0;
        return;
    }

    let probe_cmd = vec![
        "sh".to_string(),
        "-c".to_string(),
        "cat /proc/loadavg 2>/dev/null; awk '/MemTotal/{t=$2} /MemAvailable/{a=$2} END{print (t-a)*1024}' /proc/meminfo 2>/dev/null; hostname -I 2>/dev/null".to_string(),
    ];

    if let Ok(Ok((stdout, _stderr, code))) = tokio::time::timeout(
        std::time::Duration::from_millis(1500),
        ssh.exec_command(
            &rec.details.name,
            &rec.instance_dir,
            rec.details.ssh_port,
            &rec.details.ssh_user,
            &probe_cmd,
            None,
        ),
    )
    .await
    {
        if code == 0 {
            let lines: Vec<&str> = stdout.lines().collect();
            if let Some(load_line) = lines.first() {
                let load_tokens: Vec<f64> = load_line
                    .split_whitespace()
                    .take(3)
                    .filter_map(|s| s.parse::<f64>().ok())
                    .collect();
                if load_tokens.len() == 3 {
                    rec.details.load = load_tokens;
                }
            }
            if let Some(mem_line) = lines.get(1) {
                if let Ok(mem_bytes) = mem_line.trim().parse::<u64>() {
                    rec.details.memory_usage_bytes = mem_bytes;
                }
            }
            if let Some(ip_line) = lines.get(2) {
                let ips: Vec<String> = ip_line
                    .split_whitespace()
                    .filter(|ip| !ip.starts_with("127."))
                    .map(|s| s.to_string())
                    .collect();
                if !ips.is_empty() {
                    rec.details.ipv4 = ips;
                }
            }
        }
    }
}

async fn detect_host_networks() -> Vec<NetworkInterfaceInfo> {
    let mut networks = Vec::new();

    #[cfg(windows)]
    {
        let ps_cmd = r#"[System.Net.NetworkInformation.NetworkInterface]::GetAllNetworkInterfaces() | ForEach-Object { "$($_.Name)|$($_.NetworkInterfaceType)|$($_.Description)" }"#;
        if let Ok(output) = tokio::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", ps_cmd])
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 3 {
                        let name = parts[0].trim();
                        let raw_type = parts[1].trim();
                        let desc = parts[2].trim();

                        if raw_type == "24" || name.contains("Loopback") {
                            continue;
                        }

                        let net_type = match raw_type {
                            "6" => "ethernet",
                            "71" => "wireless",
                            "23" => "ppp",
                            "53" => "tunnel",
                            _ => "virtual",
                        };

                        networks.push(NetworkInterfaceInfo {
                            name: name.to_string(),
                            net_type: net_type.to_string(),
                            description: desc.to_string(),
                        });
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ifconfig").arg("-l").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for ifname in stdout.split_whitespace() {
                    if ifname.starts_with("lo")
                        || ifname.starts_with("gif")
                        || ifname.starts_with("stf")
                        || ifname.starts_with("awdl")
                        || ifname.starts_with("llw")
                        || ifname.starts_with("ap")
                    {
                        continue;
                    }
                    let net_type = if ifname.starts_with("bridge") {
                        "bridge"
                    } else if ifname.starts_with("utun") {
                        "tunnel"
                    } else {
                        "ethernet"
                    };
                    networks.push(NetworkInterfaceInfo {
                        name: ifname.to_string(),
                        net_type: net_type.to_string(),
                        description: format!("Host Network Interface {}", ifname),
                    });
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let ifname = entry.file_name().to_string_lossy().to_string();
                if ifname == "lo" {
                    continue;
                }

                let if_path = entry.path();
                let is_wireless =
                    if_path.join("wireless").exists() || if_path.join("phy80211").exists();
                let is_bridge = if_path.join("bridge").exists();

                let net_type = if is_wireless {
                    "wireless"
                } else if is_bridge {
                    "bridge"
                } else {
                    "ethernet"
                };

                networks.push(NetworkInterfaceInfo {
                    name: ifname.clone(),
                    net_type: net_type.to_string(),
                    description: format!("Host Network Interface {}", ifname),
                });
            }
        }
    }

    // Include ec2pass user-mode NAT virtual network
    networks.push(NetworkInterfaceInfo {
        name: "user-nat".to_string(),
        net_type: "user-nat".to_string(),
        description: "ec2pass User-mode NAT Virtual Network (10.0.2.0/24 with SSH port-forwarding)".to_string(),
    });

    networks
}

fn resolve_target_names(
    store: &StateStore,
    names: &[String],
    all: bool,
    default_primary: &str,
) -> Result<Vec<String>> {
    if all {
        return Ok(store.data.instances.keys().cloned().collect());
    }
    if names.is_empty() {
        let primary = store
            .data
            .settings
            .get("client.primary-name")
            .map(|s| s.as_str())
            .unwrap_or(default_primary);
        return Ok(vec![primary.to_string()]);
    }
    Ok(names.to_vec())
}

fn parse_endpoint(spec: &str) -> Option<(&str, &str)> {
    if let Some((inst, path)) = spec.split_once(':') {
        if inst.len() == 1 && inst.chars().next().unwrap().is_ascii_alphabetic() {
            return None;
        }
        if !inst.contains('/') && !inst.contains('\\') && !inst.is_empty() {
            return Some((inst, path));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_endpoint() {
        assert_eq!(parse_endpoint("inst1:/var/log"), Some(("inst1", "/var/log")));
        assert_eq!(parse_endpoint("my-vm:test.txt"), Some(("my-vm", "test.txt")));
        assert_eq!(parse_endpoint("C:\\Users\\user\\file.txt"), None);
        assert_eq!(parse_endpoint("d:/data/file.txt"), None);
        assert_eq!(parse_endpoint("/local/path/to/file"), None);
        assert_eq!(parse_endpoint("relative/path/to/file"), None);
    }

    #[test]
    fn test_is_process_alive() {
        // Current process must be alive
        let current_pid = std::process::id();
        assert!(is_process_alive(current_pid));

        // Non-existent PID must be dead
        assert!(!is_process_alive(99_999_999));
        assert!(!is_process_alive(0));
    }

    #[tokio::test]
    async fn test_detect_host_networks_has_interfaces() {
        let nets = detect_host_networks().await;
        assert!(!nets.is_empty());
        // Always includes user-nat
        assert!(nets.iter().any(|n| n.name == "user-nat"));
    }

    #[test]
    fn test_resolve_target_names_with_custom_primary() {
        let tmp = std::env::temp_dir().join(format!("ec2ps_target_test_{}", rand::random::<u32>()));
        let mut store = StateStore::load_or_create(&tmp).unwrap();

        // Default primary
        let default_targets = resolve_target_names(&store, &[], false, "primary").unwrap();
        assert_eq!(default_targets, vec!["primary".to_string()]);

        // Explicit names override default
        let explicit_targets = resolve_target_names(&store, &["vm1".to_string(), "vm2".to_string()], false, "primary").unwrap();
        assert_eq!(explicit_targets, vec!["vm1".to_string(), "vm2".to_string()]);

        // Custom primary setting
        store.set_setting("client.primary-name", "my-node").unwrap();
        let custom_targets = resolve_target_names(&store, &[], false, "primary").unwrap();
        assert_eq!(custom_targets, vec!["my-node".to_string()]);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
