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
use std::collections::BTreeMap;
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
        tracing::warn!("Failed to create data dir at {:?}: {}. Falling back to .ec2ps-data", data_dir, e);
        data_dir = PathBuf::from(".ec2ps-data");
        std::fs::create_dir_all(&data_dir)?;
    }

    let store = StateStore::load_or_create(&data_dir)?;
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

    let bind_addr = args.address.unwrap_or_else(ec2ps_protocol::default_daemon_address);

    tracing::info!(
        "ec2passd (InhausLab ec2pass Daemon v{}) listening on {} [driver: {}, data_dir: {:?}]",
        env!("CARGO_PKG_VERSION"),
        bind_addr,
        driver.driver_name(),
        data_dir
    );

    let ctx = Arc::new(DaemonContext {
        data_dir,
        store: Mutex::new(store),
        vault,
        ssh,
        driver,
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

            tracing::info!("ec2passd daemon is ready to accept client connections via Named Pipe {}", pipe_name);

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

    tracing::info!("ec2passd daemon is ready to accept client connections via TCP {}", bind_addr);

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

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let req: RpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let _ = send_frame(
                    &mut writer,
                    &RpcStreamFrame::Response(RpcResponse::Error(format!(
                        "Malformed RPC request: {}",
                        e
                    ))),
                )
                .await;
                continue;
            }
        };

        let resp = match dispatch_request(req, &ctx, &mut writer).await {
            Ok(r) => r,
            Err(e) => RpcResponse::Error(format!("{:#}", e)),
        };

        send_frame(&mut writer, &RpcStreamFrame::Response(resp)).await?;
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
                if secret != &passphrase {
                    bail!("Passphrase does not match local.passphrase");
                }
            }
            Ok(RpcResponse::OkMessage(
                "Client authenticated successfully with ec2passd.".to_string(),
            ))
        }

        RpcRequest::Find {
            search_string,
            show_unsupported: _,
            show_versions,
            force_update,
        } => {
            let mut entries = ctx.vault.fetch_catalog_extended(show_versions, force_update).await;
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
            networks: _,
            mounts,
            timeout_secs: _,
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
                Some(10),
            )
            .await?;

            let base_image_path = ctx
                .vault
                .ensure_base_image(&catalog_entry, |_msg, _pct| {})
                .await?;

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
                mounts: mount_map,
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
                &format!("Waiting for {} to boot and cloud-init to complete...", instance_name),
                Some(85),
            )
            .await?;

            let start_wait = std::time::Instant::now();
            let timeout_secs = 120u64;
            let mut is_ready = false;

            while start_wait.elapsed() < std::time::Duration::from_secs(timeout_secs) {
                let elapsed = start_wait.elapsed().as_secs();
                let pct = (85 + (elapsed as f64 / timeout_secs as f64 * 14.0) as u8).min(99);

                // 1. Check if SSH TCP port is open (1.5s timeout)
                let addr = format!("127.0.0.1:{}", ssh_port);
                let tcp_ok = match tokio::time::timeout(
                    std::time::Duration::from_millis(1500),
                    tokio::net::TcpStream::connect(&addr),
                )
                .await
                {
                    Ok(Ok(_)) => true,
                    _ => false,
                };

                if tcp_ok {
                    send_progress(
                        writer,
                        "Waiting",
                        &format!("SSH port open. Verifying cloud-init status ({}/{}s)...", elapsed, timeout_secs),
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
                        &format!("Waiting for instance to boot ({}/{}s)...", elapsed, timeout_secs),
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
                    bail!("Cannot start deleted instance '{}'. Recover it first.", name);
                }
                if rec.details.state == InstanceState::Suspended {
                    ctx.driver.resume_vm(rec).await?;
                } else if rec.details.state != InstanceState::Running {
                    let pid = ctx.driver.start_vm(rec).await?;
                    rec.pid = Some(pid);
                    let _ = ctx
                        .ssh
                        .wait_until_ready(&name, &rec.instance_dir, rec.details.ssh_port, &rec.details.ssh_user, 60)
                        .await;
                }
                rec.details.state = InstanceState::Running;
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
                    if rec.details.state == InstanceState::DelayedShutdown {
                        rec.details.state = InstanceState::Running;
                    }
                } else if let Some(mins) = delay_mins {
                    if mins > 0 {
                        rec.details.state = InstanceState::DelayedShutdown;
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
            Ok(RpcResponse::OkMessage("Purged all deleted instances.".to_string()))
        }

        RpcRequest::List { snapshots: _ } => {
            let mut store = ctx.store.lock().await;
            let mut list = Vec::with_capacity(store.data.instances.len());
            for rec in store.data.instances.values_mut() {
                if let Ok(meta) = std::fs::metadata(&rec.disk_path) {
                    rec.details.disk_usage_bytes = meta.len();
                }
                if rec.details.state != InstanceState::Running {
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
                bail!("Instance '{}' is not running (current state: {})", name, state);
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

            let mut store = ctx.store.lock().await;
            if !store.data.instances.contains_key(&instance) {
                bail!("No such instance: '{}'", instance);
            }

            store.add_mount(
                &instance,
                abs_src,
                target_path,
                uid_map,
                gid_map,
                mount_type,
            )?;
            Ok(RpcResponse::OkMessage("Mounted".to_string()))
        }

        RpcRequest::Umount {
            instance,
            target_path,
        } => {
            let mut store = ctx.store.lock().await;
            store.remove_mount(&instance, target_path.as_deref())?;
            Ok(RpcResponse::OkMessage("Unmounted".to_string()))
        }

        RpcRequest::Transfer {
            sources,
            destination,
            recursive,
            parents: _,
        } => {
            // Find which instance is referenced in either sources or destination (`instance:path`)
            let find_inst = |s: &str| -> Option<String> {
                if let Some((inst, _path)) = s.split_once(':') {
                    if inst.len() > 1 {
                        return Some(inst.to_string());
                    }
                }
                None
            };

            let inst_name = find_inst(&destination)
                .or_else(|| sources.iter().find_map(|s| find_inst(s)))
                .context("Transfer requires at least one instance path in the form <instance>:<path>")?;

            let (instance_dir, ssh_port, ssh_user) = {
                let store = ctx.store.lock().await;
                let rec = store
                    .data
                    .instances
                    .get(&inst_name)
                    .with_context(|| format!("Instance '{}' does not exist", inst_name))?;
                (
                    rec.instance_dir.clone(),
                    rec.details.ssh_port,
                    rec.details.ssh_user.clone(),
                )
            };

            for src in &sources {
                ctx.ssh
                    .transfer_path(
                        &instance_dir,
                        ssh_port,
                        &ssh_user,
                        src,
                        &destination,
                        recursive,
                    )
                    .await?;
            }
            Ok(RpcResponse::OkMessage("Transfer completed".to_string()))
        }

        RpcRequest::Snapshot {
            instance,
            snapshot_name,
            comment,
        } => {
            let mut store = ctx.store.lock().await;
            let snap = store.create_snapshot(&instance, snapshot_name, comment)?;
            let disk_path = store.data.instances[&instance].disk_path.clone();
            ctx.driver.take_disk_snapshot(&disk_path, &snap.name).await?;
            Ok(RpcResponse::SnapshotCreated(snap))
        }

        RpcRequest::Restore {
            instance,
            snapshot_name,
            destructive: _,
        } => {
            let mut store = ctx.store.lock().await;
            let snap = store.restore_snapshot(&instance, &snapshot_name)?;
            let disk_path = store.data.instances[&instance].disk_path.clone();
            ctx.driver
                .restore_disk_snapshot(&disk_path, &snap.name)
                .await?;
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

            let (ssh_port, qmp_port) = store.allocate_ports();
            let guest_ip = store.allocate_guest_ip();
            let mac = store.allocate_mac();

            let mut dst_rec = src_rec.clone();
            dst_rec.details.name = dst_name.clone();
            dst_rec.details.ipv4 = vec![guest_ip];
            dst_rec.details.ssh_port = ssh_port;
            dst_rec.details.snapshots.clear();
            dst_rec.details.created_at = Utc::now();
            dst_rec.instance_dir = dst_dir.clone();
            dst_rec.disk_path = dst_dir.join("al2023-root.qcow2");
            dst_rec.seed_iso_path = dst_dir.join("seed.iso");
            dst_rec.qmp_port = qmp_port;
            dst_rec.mac_address = mac;

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
                    .filter(|(key_name, _)| key_name.starts_with(&prefix) && !key_name.starts_with("local._"))
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
        "cat /proc/loadavg 2>/dev/null; awk '/MemTotal/{t=$2} /MemAvailable/{a=$2} END{print (t-a)*1024}' /proc/meminfo 2>/dev/null".to_string(),
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

    #[cfg(unix)]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let ifname = entry.file_name().to_string_lossy().to_string();
                if ifname == "lo" {
                    continue;
                }

                let if_path = entry.path();
                let is_wireless = if_path.join("wireless").exists() || if_path.join("phy80211").exists();
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

    // Always include ec2pass user-mode NAT virtual network
    networks.push(NetworkInterfaceInfo {
        name: "ec2psbr0".to_string(),
        net_type: "user-nat".to_string(),
        description: "ec2pass User-mode NAT Virtual Network (10.23.104.0/24)".to_string(),
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
