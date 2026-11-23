//! ec2pass CLI for Amazon Linux 2023.
//!
//! Complete command-line interface supporting:
//! `launch`, `start`, `stop`, `restart`, `suspend`, `delete`, `recover`, `purge`,
//! `shell`, `exec`, `info`, `list`, `find`, `mount`, `umount`, `transfer`,
//! `snapshot`, `restore`, `clone`, `get`, `set`, `alias`, `unalias`, `aliases`,
//! `authenticate`, `networks`, `version`.
//!
//! Automatically spawns `ec2passd` in the background if the daemon is not already running.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use ec2ps_protocol::{
    format_catalog_find, format_instance_info, format_instance_list, parse_byte_size,
    render_ascii_table, render_progress_bar, OutputFormat, RpcRequest, RpcResponse, RpcStreamFrame,
};
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[derive(Parser, Debug)]
#[command(
    name = "ec2pass",
    version,
    about = "InhausLab ec2pass - Virtual Machine Orchestrator for Amazon Linux 2023"
)]
struct Cli {
    /// Daemon RPC endpoint address
    #[arg(
        long = "daemon-addr",
        alias = "daemon-address",
        global = true,
        env = "EC2PASS_DAEMON_ADDR",
        hide = true
    )]
    daemon_addr: Option<String>,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create and start a new Amazon Linux 2023 instance
    Launch {
        /// Image to launch (e.g. al2023, 2023, default, minimal, kernel-6.12, al2, or custom URL/path)
        image: Option<String>,

        /// Specific Amazon Linux 2023 release version from AWS CDN (e.g. 2023.12.20261119.0). Defaults to latest.
        #[arg(short = 'V', long = "image-version", visible_alias = "ver")]
        image_version: Option<String>,

        /// Name for the instance (auto-generates a petname if omitted)
        #[arg(short, long)]
        name: Option<String>,

        /// Number of CPUs to allocate (Minimum: 1, Default: 1)
        #[arg(short, long, default_value_t = 1)]
        cpus: u32,

        /// Amount of memory to allocate (Minimum: 128M, Default: 1G)
        #[arg(short, long, default_value = "1G")]
        memory: String,

        /// Disk space to allocate (Minimum: 2G, Default: 5G)
        #[arg(short, long, default_value = "5G")]
        disk: String,

        /// Path or URL to a cloud-init cloud-config YAML file (or '-' for stdin)
        #[arg(long = "cloud-init")]
        cloud_init: Option<String>,

        /// Add a network interface to the instance
        #[arg(long = "network")]
        networks: Vec<String>,

        /// Mount a local directory inside the instance (<local-path>:<instance-path>)
        #[arg(long = "mount")]
        mounts: Vec<String>,

        /// Maximum time (in seconds) to wait for the command to complete
        #[arg(long, default_value_t = 300)]
        timeout: u64,
    },

    /// Start instances
    Start {
        /// Names of instances to start (defaults to primary instance if omitted)
        names: Vec<String>,

        /// Start all instances
        #[arg(long)]
        all: bool,
    },

    /// Stop running instances
    Stop {
        /// Names of instances to stop
        names: Vec<String>,

        /// Stop all instances
        #[arg(long)]
        all: bool,

        /// Force the instance(s) to shut down immediately
        #[arg(long)]
        force: bool,

        /// Time from now, in minutes, to delay shutdown of the instance
        #[arg(short = 't', long = "time")]
        time: Option<u32>,

        /// Cancel a pending delayed shutdown
        #[arg(short = 'c', long = "cancel")]
        cancel: bool,
    },

    /// Restart instances
    Restart {
        /// Names of instances to restart
        names: Vec<String>,

        /// Restart all instances
        #[arg(long)]
        all: bool,
    },

    /// Suspend running instances
    Suspend {
        /// Names of instances to suspend
        names: Vec<String>,

        /// Suspend all instances
        #[arg(long)]
        all: bool,
    },

    /// Delete instances (to be purged with the "purge" command, or immediately with --purge)
    Delete {
        /// Names of instances to delete
        names: Vec<String>,

        /// Delete all instances
        #[arg(long)]
        all: bool,

        /// Purge instances immediately
        #[arg(short, long)]
        purge: bool,
    },

    /// Recover deleted instances
    Recover {
        /// Names of instances to recover
        names: Vec<String>,

        /// Recover all deleted instances
        #[arg(long)]
        all: bool,
    },

    /// Purge all deleted instances permanently
    Purge,

    /// List all available instances or snapshots
    List {
        /// Output list in the requested format (table, json, csv, yaml)
        #[arg(long, default_value = "table")]
        format: OutputFormat,

        /// List all available snapshots instead of instances
        #[arg(long)]
        snapshots: bool,
    },

    /// Display information about instances or snapshots
    Info {
        /// Names of instances to display information about
        names: Vec<String>,

        /// Display info for all instances
        #[arg(long)]
        all: bool,

        /// Display detailed info about snapshots
        #[arg(long)]
        snapshots: bool,

        /// Output info in the requested format (table, json, csv, yaml)
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },

    /// Display available Amazon Linux 2023 images to create instances from
    Find {
        /// Optional query string to filter images
        search_string: Option<String>,

        /// Show all available Amazon Linux 2023 release versions from AWS CDN
        #[arg(short = 'V', long = "show-versions", visible_alias = "all-versions")]
        show_versions: bool,

        /// Show unsupported cloud images as well
        #[arg(long)]
        show_unsupported: bool,

        /// Force the image catalog to update from Amazon Linux 2023 CDN
        #[arg(long)]
        force_update: bool,

        /// Output find results in the requested format (table, json, csv, yaml)
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },

    /// Run a command on an instance
    Exec {
        /// Name of the target instance
        name: String,

        /// Change to this directory inside the instance before execution
        #[arg(short = 'd', long = "working-directory")]
        working_directory: Option<String>,

        /// Do not map the host working directory to a mounted path
        #[arg(short = 'n', long = "no-map-working-directory")]
        no_map_working_directory: bool,

        /// Command and arguments to execute inside the Amazon Linux 2023 instance
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
        command: Vec<String>,
    },

    /// Open a shell on a running instance (auto-launches primary if it does not exist)
    Shell {
        /// Name of the instance to open a shell on (defaults to primary)
        name: Option<String>,
    },

    /// Mount a local directory in the instance
    Mount {
        /// Path of the local directory to mount
        source: String,

        /// Target mount points in <name>[:<path>] format
        #[arg(required = true)]
        targets: Vec<String>,

        /// A mapping of user IDs (<host>:<instance>)
        #[arg(short = 'u', long = "uid-map")]
        uid_map: Vec<String>,

        /// A mapping of group IDs (<host>:<instance>)
        #[arg(short = 'g', long = "gid-map")]
        gid_map: Vec<String>,

        /// Specify the type of mount to use (classic / native)
        #[arg(short = 't', long = "type", default_value = "native")]
        mount_type: String,
    },

    /// Unmount a directory from an instance
    Umount {
        /// Mount points in <name>[:<path>] format to unmount
        #[arg(required = true)]
        mounts: Vec<String>,
    },

    /// Transfer files between the host and instances
    Transfer {
        /// One or more paths to transfer, prefixed with <name>: for paths inside the instance, and the destination path last
        #[arg(required = true, num_args = 2..)]
        paths: Vec<String>,

        /// Recursively copy entire directories
        #[arg(short, long)]
        recursive: bool,

        /// Make parent directories as needed
        #[arg(short, long)]
        parents: bool,
    },

    /// Take a snapshot of an instance
    Snapshot {
        /// The instance to take a snapshot of
        instance: String,

        /// Optional name for the snapshot (defaults to snapshot1, snapshot2, ...)
        #[arg(short, long)]
        name: Option<String>,

        /// Optional comment describing the snapshot
        #[arg(short, long, alias = "description")]
        comment: Option<String>,
    },

    /// Restore an instance from a snapshot
    Restore {
        /// Target instance or snapshot specification in <instance>.<snapshot> or <instance> format
        target: String,

        /// Snapshot name to restore when targeting an instance
        #[arg(short, long)]
        snapshot: Option<String>,

        /// Discard the current state without taking an automatic backup snapshot
        #[arg(short, long)]
        destructive: bool,
    },

    /// Clone an existing stopped instance
    Clone {
        /// The source instance to clone
        source_name: String,

        /// Optional custom name for the cloned instance
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Get the configuration setting corresponding to the given key, or all settings if no key is specified
    Get {
        /// Settings key (e.g. client.primary-name, local.driver, local.<instance>.cpus)
        key: Option<String>,

        /// Output the settings value alone, without the key name
        #[arg(long)]
        raw: bool,

        /// List available settings keys
        #[arg(long)]
        keys: bool,
    },

    /// Set, to the given value, the configuration setting corresponding to the given key
    Set {
        /// Setting key or key=value pair (e.g. local.driver=qemu)
        key: String,

        /// Optional value if key and value are passed as separate arguments
        value: Option<String>,
    },

    /// Create an alias to be executed on a given instance
    Alias {
        /// Definition in <instance>:<command> format
        definition: String,

        /// Optional name for the alias (defaults to the command name)
        alias_name: Option<String>,

        /// Do not map the host execution working directory
        #[arg(short = 'n', long = "no-map-working-directory")]
        no_map_working_directory: bool,
    },

    /// Remove an alias
    Unalias {
        /// Names of aliases to remove
        names: Vec<String>,

        /// Remove all aliases
        #[arg(long)]
        all: bool,
    },

    /// List available aliases
    Aliases {
        /// Output format (table, json, csv, yaml)
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },

    /// Authenticate client with the daemon
    Authenticate {
        /// Passphrase configured in local.passphrase
        passphrase: String,
    },

    /// List available network interfaces
    Networks {
        /// Output format (table, json, csv, yaml)
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },

    /// Show version details for the client and daemon
    Version {
        /// Output format (table, json, csv, yaml)
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },
}

pub async fn entry_main() {
    if let Err(err) = run_cli().await {
        eprintln!("error: {:#}", err);
        std::process::exit(1);
    }
}

async fn run_cli() -> Result<()> {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            // Check if the first non-option argument matches a registered alias
            let args: Vec<String> = std::env::args().collect();
            let mut candidate_idx = 1;
            let mut custom_addr = None;
            while candidate_idx < args.len() {
                if args[candidate_idx] == "--daemon-addr" && candidate_idx + 1 < args.len() {
                    custom_addr = Some(args[candidate_idx + 1].clone());
                    candidate_idx += 2;
                } else if args[candidate_idx].starts_with("--daemon-addr=") {
                    custom_addr = Some(
                        args[candidate_idx]
                            .trim_start_matches("--daemon-addr=")
                            .to_string(),
                    );
                    candidate_idx += 1;
                } else if args[candidate_idx].starts_with('-') {
                    candidate_idx += 1;
                } else {
                    break;
                }
            }

            if candidate_idx < args.len() {
                let candidate_alias = &args[candidate_idx];
                let default_addr = ec2ps_protocol::default_daemon_address();
                let addr = custom_addr.as_deref().unwrap_or(&default_addr);

                if let Ok(RpcResponse::AliasList(aliases)) =
                    send_rpc(addr, RpcRequest::Aliases, false).await
                {
                    if let Some(target_alias) = aliases
                        .into_iter()
                        .find(|a| a.alias_name == *candidate_alias)
                    {
                        let mut exec_cmd = vec![target_alias.command];
                        exec_cmd.extend(args.into_iter().skip(candidate_idx + 1));
                        let wd = if target_alias.working_directory.is_empty()
                            || target_alias.working_directory == "map"
                        {
                            None
                        } else {
                            Some(target_alias.working_directory)
                        };

                        let resp = send_rpc(
                            addr,
                            RpcRequest::Exec {
                                name: target_alias.instance,
                                command: exec_cmd,
                                working_dir: wd,
                            },
                            false,
                        )
                        .await?;

                        if let RpcResponse::ExecResult {
                            stdout,
                            stderr,
                            exit_code,
                        } = resp
                        {
                            if !stdout.is_empty() {
                                print!("{}", stdout);
                            }
                            if !stderr.is_empty() {
                                eprint!("{}", stderr);
                            }
                            std::process::exit(exit_code);
                        }
                    }
                }
            }

            // If not an alias or alias execution failed, print standard clap error and exit
            e.exit();
        }
    };
    let default_addr = ec2ps_protocol::default_daemon_address();
    let addr = cli.daemon_addr.as_deref().unwrap_or(&default_addr);

    match cli.command {
        Commands::Version { format } => {
            let resp = send_rpc(addr, RpcRequest::Version, false).await?;
            if let RpcResponse::Version {
                client_version,
                daemon_version,
                driver: _,
                default_image: _,
            } = resp
            {
                match format {
                    OutputFormat::Table => {
                        println!("{:<14}{}", "ec2pass", client_version);
                        println!("{:<14}{}", "ec2passd", daemon_version);
                    }
                    OutputFormat::Json => {
                        let json = serde_json::json!({
                            "ec2pass": client_version,
                            "ec2passd": daemon_version,
                        });
                        println!("{}", serde_json::to_string_pretty(&json)?);
                    }
                    OutputFormat::Yaml => {
                        println!("ec2pass: {}\nec2passd: {}", client_version, daemon_version);
                    }
                    OutputFormat::Csv => {
                        println!(
                            "Component,Version\nec2pass,{}\nec2passd,{}",
                            client_version, daemon_version
                        );
                    }
                }
            }
        }

        Commands::Find {
            search_string,
            show_versions,
            show_unsupported,
            force_update,
            format,
        } => {
            let resp = send_rpc(
                addr,
                RpcRequest::Find {
                    search_string,
                    show_unsupported,
                    show_versions,
                    force_update,
                },
                true,
            )
            .await?;
            if let RpcResponse::CatalogList(entries) = resp {
                print!("{}", format_catalog_find(&entries, format)?);
            }
        }

        Commands::Launch {
            image,
            image_version,
            name,
            cpus,
            memory,
            disk,
            cloud_init,
            networks,
            mounts,
            timeout,
        } => {
            // Hypervisor prerequisite check: fail immediately with actionable guide if missing
            ec2ps_core::hypervisor::check_hypervisor_prerequisites()?;

            let memory_bytes = parse_byte_size(&memory)?;
            let disk_bytes = parse_byte_size(&disk)?;

            let cloud_init_yaml =
                if let Some(ci_path) = cloud_init {
                    if ci_path == "-" {
                        use std::io::Read;
                        let mut buf = String::new();
                        io::stdin().read_to_string(&mut buf)?;
                        Some(buf)
                    } else {
                        Some(std::fs::read_to_string(&ci_path).with_context(|| {
                            format!("Failed to read cloud-init file '{}'", ci_path)
                        })?)
                    }
                } else {
                    None
                };

            let mut parsed_mounts = Vec::new();
            for m in mounts {
                if let Some((src, dst)) = split_host_guest_mount(&m) {
                    parsed_mounts.push((src, dst));
                } else {
                    bail!(
                        "Invalid --mount specification '{}'. Expected <local-path>:<instance-path>",
                        m
                    );
                }
            }

            let resp = send_rpc(
                addr,
                RpcRequest::Launch {
                    image,
                    image_version,
                    name,
                    cpus,
                    memory_bytes,
                    disk_bytes,
                    cloud_init_yaml,
                    networks,
                    mounts: parsed_mounts,
                    timeout_secs: timeout,
                },
                true,
            )
            .await?;

            if let RpcResponse::Launched(details) = resp {
                println!("Launched: {}", details.name);
            }
        }

        Commands::Start { names, all } => {
            ec2ps_core::hypervisor::check_hypervisor_prerequisites()?;
            send_rpc(addr, RpcRequest::Start { names, all }, true).await?;
        }

        Commands::Stop {
            names,
            all,
            force,
            time,
            cancel,
        } => {
            send_rpc(
                addr,
                RpcRequest::Stop {
                    names,
                    all,
                    force,
                    delay_mins: time,
                    cancel_shutdown: cancel,
                },
                true,
            )
            .await?;
        }

        Commands::Restart { names, all } => {
            send_rpc(addr, RpcRequest::Restart { names, all }, true).await?;
        }

        Commands::Suspend { names, all } => {
            send_rpc(addr, RpcRequest::Suspend { names, all }, true).await?;
        }

        Commands::Delete { names, all, purge } => {
            send_rpc(addr, RpcRequest::Delete { names, all, purge }, true).await?;
        }

        Commands::Recover { names, all } => {
            send_rpc(addr, RpcRequest::Recover { names, all }, true).await?;
        }

        Commands::Purge => {
            send_rpc(addr, RpcRequest::Purge, true).await?;
        }

        Commands::List { format, snapshots } => {
            let resp = send_rpc(addr, RpcRequest::List { snapshots }, false).await?;
            if let RpcResponse::InstanceList(instances) = resp {
                print!("{}", format_instance_list(&instances, snapshots, format)?);
            }
        }

        Commands::Info {
            names,
            all,
            snapshots,
            format,
        } => {
            let resp = send_rpc(
                addr,
                RpcRequest::Info {
                    names,
                    all,
                    snapshots,
                },
                false,
            )
            .await?;
            if let RpcResponse::InstanceList(instances) = resp {
                if snapshots {
                    print!("{}", format_instance_list(&instances, true, format)?);
                } else {
                    print!("{}", format_instance_info(&instances, format)?);
                }
            }
        }

        Commands::Exec {
            name,
            working_directory,
            no_map_working_directory: _,
            command,
        } => {
            let resp = send_rpc(
                addr,
                RpcRequest::Exec {
                    name,
                    command,
                    working_dir: working_directory,
                },
                false,
            )
            .await?;
            if let RpcResponse::ExecResult {
                stdout,
                stderr,
                exit_code,
            } = resp
            {
                if !stdout.is_empty() {
                    print!("{}", stdout);
                }
                if !stderr.is_empty() {
                    eprint!("{}", stderr);
                }
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
            }
        }

        Commands::Shell { name } => {
            // Check if instance exists; if not and it's default primary, auto-launch it
            let shell_resp =
                match send_rpc(addr, RpcRequest::ShellInfo { name: name.clone() }, false).await {
                    Ok(r) => r,
                    Err(_) if name.is_none() || name.as_deref() == Some("primary") => {
                        let target = name.unwrap_or_else(|| "primary".to_string());
                        println!(
                            "Launching primary instance '{}' with Amazon Linux 2023...",
                            target
                        );
                        send_rpc(
                            addr,
                            RpcRequest::Launch {
                                image: Some("al2023".to_string()),
                                image_version: None,
                                name: Some(target.clone()),
                                cpus: 1,
                                memory_bytes: 1024 * 1024 * 1024,
                                disk_bytes: 5 * 1024 * 1024 * 1024,
                                cloud_init_yaml: None,
                                networks: vec![],
                                mounts: vec![],
                                timeout_secs: 300,
                            },
                            true,
                        )
                        .await?;
                        send_rpc(addr, RpcRequest::ShellInfo { name: Some(target) }, false).await?
                    }
                    Err(e) => return Err(e),
                };

            if let RpcResponse::ShellConnection {
                instance: _,
                ssh_host,
                ssh_port,
                ssh_user,
                ssh_key_path,
            } = shell_resp
            {
                let status = std::process::Command::new("ssh")
                    .args([
                        "-i",
                        &ssh_key_path,
                        "-p",
                        &ssh_port.to_string(),
                        "-o",
                        "StrictHostKeyChecking=no",
                        "-o",
                        "UserKnownHostsFile=/dev/null",
                        &format!("{}@{}", ssh_user, ssh_host),
                    ])
                    .status()
                    .context("Failed to launch interactive ssh client")?;
                if !status.success() {
                    std::process::exit(status.code().unwrap_or(1));
                }
            }
        }

        Commands::Mount {
            source,
            targets,
            uid_map,
            gid_map,
            mount_type,
        } => {
            let parsed_uids = parse_id_mappings(&uid_map)?;
            let parsed_gids = parse_id_mappings(&gid_map)?;

            for t in targets {
                let (inst, target_path) = match t.split_once(':') {
                    Some((i, p)) => (i.to_string(), p.to_string()),
                    None => (
                        t.clone(),
                        format!(
                            "/home/ec2-user/{}",
                            PathBuf::from(&source)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                        ),
                    ),
                };
                send_rpc(
                    addr,
                    RpcRequest::Mount {
                        source_path: source.clone(),
                        instance: inst,
                        target_path,
                        uid_map: parsed_uids.clone(),
                        gid_map: parsed_gids.clone(),
                        mount_type: mount_type.clone(),
                    },
                    true,
                )
                .await?;
            }
        }

        Commands::Umount { mounts } => {
            for m in mounts {
                let (inst, target_path) = match m.split_once(':') {
                    Some((i, p)) => (i.to_string(), Some(p.to_string())),
                    None => (m, None),
                };
                send_rpc(
                    addr,
                    RpcRequest::Umount {
                        instance: inst,
                        target_path,
                    },
                    true,
                )
                .await?;
            }
        }

        Commands::Transfer {
            mut paths,
            recursive,
            parents,
        } => {
            let mut destination = paths.pop().context("Missing destination path")?;
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            let resolve_path = |p: &str| -> String {
                if p.contains(':') {
                    p.to_string()
                } else {
                    let pb = PathBuf::from(p);
                    if pb.is_absolute() {
                        pb.to_string_lossy().to_string()
                    } else {
                        cwd.join(pb).to_string_lossy().to_string()
                    }
                }
            };

            let resolved_sources: Vec<String> = paths.iter().map(|p| resolve_path(p)).collect();
            destination = resolve_path(&destination);

            send_rpc(
                addr,
                RpcRequest::Transfer {
                    sources: resolved_sources,
                    destination,
                    recursive,
                    parents,
                },
                true,
            )
            .await?;
        }

        Commands::Snapshot {
            instance,
            name,
            comment,
        } => {
            let resp = send_rpc(
                addr,
                RpcRequest::Snapshot {
                    instance: instance.clone(),
                    snapshot_name: name,
                    comment,
                },
                true,
            )
            .await?;
            if let RpcResponse::SnapshotCreated(snap) = resp {
                println!("Snapshot taken: {}.{}", instance, snap.name);
            }
        }

        Commands::Restore {
            target,
            snapshot,
            destructive,
        } => {
            let (instance, snapshot_name) = if let Some(s) = snapshot {
                (target.as_str(), s)
            } else if let Some((inst, snap)) = target.split_once('.') {
                (inst, snap.to_string())
            } else {
                bail!("Restore target must be in <instance>.<snapshot> format or specify --snapshot / -s");
            };
            let resp = send_rpc(
                addr,
                RpcRequest::Restore {
                    instance: instance.to_string(),
                    snapshot_name,
                    destructive,
                },
                true,
            )
            .await?;
            if let RpcResponse::SnapshotCreated(snap) = resp {
                println!("Restored: {}.{}", instance, snap.name);
            }
        }

        Commands::Clone { source_name, name } => {
            let resp = send_rpc(
                addr,
                RpcRequest::Clone {
                    source_name: source_name.clone(),
                    destination_name: name,
                },
                true,
            )
            .await?;
            if let RpcResponse::Launched(details) = resp {
                println!("Cloned from {} to {}.", source_name, details.name);
            }
        }

        Commands::Get { key, raw, keys } => {
            let resp = send_rpc(
                addr,
                RpcRequest::Get {
                    key,
                    keys_only: keys,
                },
                false,
            )
            .await?;
            if let RpcResponse::SettingsMap(map) = resp {
                if keys {
                    for k in map.keys() {
                        println!("{}", k);
                    }
                } else if raw && map.len() == 1 {
                    if let Some(v) = map.values().next() {
                        println!("{}", v);
                    }
                } else {
                    for (k, v) in map {
                        println!("{}={}", k, v);
                    }
                }
            }
        }

        Commands::Set { key, value } => {
            let (k, v) = if let Some(val) = value {
                (key, val)
            } else if let Some((k, v)) = key.split_once('=') {
                (k.to_string(), v.to_string())
            } else {
                bail!("Invalid format. Use <key>=<value> or <key> <value>");
            };
            send_rpc(
                addr,
                RpcRequest::Set {
                    key: k.trim().to_string(),
                    value: v.trim().to_string(),
                },
                false,
            )
            .await?;
        }

        Commands::Alias {
            definition,
            alias_name,
            no_map_working_directory,
        } => {
            let (inst, cmd) = definition
                .split_once(':')
                .context("Alias definition must be in <instance>:<command> format")?;
            let wd = if no_map_working_directory {
                "default".to_string()
            } else {
                "map".to_string()
            };
            send_rpc(
                addr,
                RpcRequest::Alias {
                    instance: inst.to_string(),
                    command: cmd.to_string(),
                    alias_name,
                    working_directory: wd,
                },
                false,
            )
            .await?;
        }

        Commands::Unalias { names, all } => {
            if names.is_empty() && !all {
                bail!("Please specify an alias name or --all");
            }
            send_rpc(
                addr,
                RpcRequest::Unalias {
                    alias_names: names,
                    all,
                },
                false,
            )
            .await?;
        }

        Commands::Aliases { format } => {
            let resp = send_rpc(addr, RpcRequest::Aliases, false).await?;
            if let RpcResponse::AliasList(list) = resp {
                match format {
                    OutputFormat::Table => {
                        if list.is_empty() {
                            println!("No aliases defined.");
                        } else {
                            let rows: Vec<Vec<String>> = list
                                .iter()
                                .map(|a| {
                                    vec![
                                        a.alias_name.clone(),
                                        a.instance.clone(),
                                        a.command.clone(),
                                        a.working_directory.clone(),
                                    ]
                                })
                                .collect();
                            print!(
                                "{}",
                                render_ascii_table(
                                    &["Alias", "Instance", "Command", "Working directory"],
                                    &rows
                                )
                            );
                        }
                    }
                    OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&list)?),
                    OutputFormat::Yaml => print!("{}", serde_yaml::to_string(&list)?),
                    OutputFormat::Csv => {
                        println!("Alias,Instance,Command,Working directory");
                        for a in list {
                            println!(
                                "{},{},{},{}",
                                a.alias_name, a.instance, a.command, a.working_directory
                            );
                        }
                    }
                }
            }
        }

        Commands::Authenticate { passphrase } => {
            send_rpc(addr, RpcRequest::Authenticate { passphrase }, false).await?;
        }

        Commands::Networks { format } => {
            let resp = send_rpc(addr, RpcRequest::Networks, false).await?;
            if let RpcResponse::NetworkList(nets) = resp {
                match format {
                    OutputFormat::Table => {
                        let rows: Vec<Vec<String>> = nets
                            .iter()
                            .map(|n| {
                                vec![n.name.clone(), n.net_type.clone(), n.description.clone()]
                            })
                            .collect();
                        print!(
                            "{}",
                            render_ascii_table(&["Name", "Type", "Description"], &rows)
                        );
                    }
                    OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&nets)?),
                    OutputFormat::Yaml => print!("{}", serde_yaml::to_string(&nets)?),
                    OutputFormat::Csv => {
                        println!("Name,Type,Description");
                        for n in nets {
                            println!("{},{},{}", n.name, n.net_type, n.description);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Connects to `ec2passd` over TCP or Windows Named Pipe. If the daemon is not yet running,
/// automatically spawns `ec2passd` in the background and waits for it to accept connections.
async fn connect_or_spawn_daemon(addr: &str) -> Result<TcpStream> {
    if let Ok(stream) = TcpStream::connect(addr).await {
        return Ok(stream);
    }

    // Attempt to find `ec2passd` binary next to current executable
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(bin_dir) = current_exe.parent() {
            let daemon_bin_name = if cfg!(windows) {
                "ec2passd.exe"
            } else {
                "ec2passd"
            };
            let mut daemon_path = bin_dir.join(daemon_bin_name);
            if !daemon_path.exists() {
                let fallback = if cfg!(windows) { "mpd.exe" } else { "mpd" };
                daemon_path = bin_dir.join(fallback);
            }
            if !daemon_path.exists() {
                let fallback = if cfg!(windows) {
                    "alpassd.exe"
                } else {
                    "alpassd"
                };
                daemon_path = bin_dir.join(fallback);
            }
            if daemon_path.exists() {
                let _ = std::process::Command::new(&daemon_path)
                    .arg("--address")
                    .arg(addr)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();

                for _ in 0..50 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    if let Ok(stream) = TcpStream::connect(addr).await {
                        return Ok(stream);
                    }
                }
            }
        }
    }

    bail!(
        "Cannot connect to ec2passd daemon at {}. Start the daemon with `ec2passd`.",
        addr
    )
}

#[cfg(windows)]
async fn connect_or_spawn_pipe_daemon(
    pipe_name: &str,
) -> Result<tokio::net::windows::named_pipe::NamedPipeClient> {
    use tokio::net::windows::named_pipe::ClientOptions;

    // Fast check if daemon is already running
    if let Ok(client) = ClientOptions::new().open(pipe_name) {
        return Ok(client);
    }

    // Attempt to spawn daemon
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(bin_dir) = current_exe.parent() {
            let daemon_path = bin_dir.join("ec2passd.exe");
            if daemon_path.exists() {
                let _ = std::process::Command::new(&daemon_path)
                    .arg("--address")
                    .arg(pipe_name)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();

                for _ in 0..50 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    if let Ok(client) = ClientOptions::new().open(pipe_name) {
                        return Ok(client);
                    }
                }
            }
        }
    }

    bail!(
        "Cannot connect to ec2passd daemon at {}. Start the daemon with `ec2passd`.",
        pipe_name
    )
}

async fn send_rpc(addr: &str, req: RpcRequest, show_progress: bool) -> Result<RpcResponse> {
    if addr.starts_with(r"\\.\pipe\") || addr.starts_with("npipe:") {
        #[cfg(windows)]
        {
            let pipe_name = if addr.starts_with("npipe:") {
                addr.trim_start_matches("npipe:")
            } else {
                addr
            };
            let stream = connect_or_spawn_pipe_daemon(pipe_name).await?;
            return send_rpc_stream(stream, req, show_progress).await;
        }
        #[cfg(not(windows))]
        bail!("Named pipes are only supported on Windows");
    }

    let stream = connect_or_spawn_daemon(addr).await?;
    send_rpc_stream(stream, req, show_progress).await
}

fn default_prompt_for_req(req: &RpcRequest) -> String {
    match req {
        RpcRequest::Launch { .. } => "Launching instance...".to_string(),
        RpcRequest::Start { names, .. } => {
            if names.is_empty() {
                "Starting instance...".to_string()
            } else {
                format!("Starting {}...", names.join(", "))
            }
        }
        RpcRequest::Stop { names, .. } => {
            if names.is_empty() {
                "Stopping instance...".to_string()
            } else {
                format!("Stopping {}...", names.join(", "))
            }
        }
        RpcRequest::Restart { names, .. } => {
            if names.is_empty() {
                "Restarting instance...".to_string()
            } else {
                format!("Restarting {}...", names.join(", "))
            }
        }
        RpcRequest::Suspend { names, .. } => {
            if names.is_empty() {
                "Suspending instance...".to_string()
            } else {
                format!("Suspending {}...", names.join(", "))
            }
        }
        RpcRequest::Mount {
            instance,
            target_path,
            ..
        } => {
            format!("Mounting directory into {}:{}...", instance, target_path)
        }
        RpcRequest::Umount {
            instance,
            target_path,
        } => {
            if let Some(p) = target_path {
                format!("Unmounting {}:{}...", instance, p)
            } else {
                format!("Unmounting all mounts on {}...", instance)
            }
        }
        RpcRequest::Transfer { destination, .. } => {
            format!("Transferring files => {}...", destination)
        }
        RpcRequest::Snapshot {
            instance,
            snapshot_name,
            ..
        } => {
            if let Some(name) = snapshot_name {
                format!("Taking snapshot '{}' on {}...", name, instance)
            } else {
                format!("Taking snapshot on {}...", instance)
            }
        }
        RpcRequest::Restore {
            instance,
            snapshot_name,
            ..
        } => {
            format!("Restoring snapshot '{}' on {}...", snapshot_name, instance)
        }
        RpcRequest::Clone {
            source_name,
            destination_name,
        } => {
            if let Some(dest) = destination_name {
                format!("Cloning {} to {}...", source_name, dest)
            } else {
                format!("Cloning {}...", source_name)
            }
        }
        RpcRequest::Delete { names, purge, .. } => {
            let action = if *purge { "Purging" } else { "Deleting" };
            format!("{} {}...", action, names.join(", "))
        }
        RpcRequest::Recover { names, .. } => {
            format!("Recovering {}...", names.join(", "))
        }
        RpcRequest::Purge => "Purging all deleted instances...".to_string(),
        _ => "Working...".to_string(),
    }
}

async fn send_rpc_stream<S>(stream: S, req: RpcRequest, show_progress: bool) -> Result<RpcResponse>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (reader, mut writer) = tokio::io::split(stream);

    let mut payload = serde_json::to_string(&req)?;
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await?;
    writer.flush().await?;

    let is_terminal = io::stderr().is_terminal();
    let mut last_was_inline = false;
    let spinner_frames = ["|", "/", "-", "\\"];
    let mut spinner_idx = 0usize;
    let mut current_status = default_prompt_for_req(&req);
    let mut retrieving_progress: Option<(u8, String)> = None;

    let mut interval = tokio::time::interval(Duration::from_millis(100));
    let mut lines = BufReader::new(reader).lines();

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if is_terminal && show_progress {
                    let frame = spinner_frames[spinner_idx % spinner_frames.len()];
                    spinner_idx = spinner_idx.wrapping_add(1);

                    if let Some((pct, ref msg)) = retrieving_progress {
                        let bar = render_progress_bar(pct, 20);
                        eprint!("\r\x1b[2K{} [Retrieving image] {} {:>3}%  {}", frame, bar, pct, msg);
                    } else {
                        eprint!("\r\x1b[2K{} {}", frame, current_status);
                    }
                    let _ = io::stderr().flush();
                    last_was_inline = true;
                }
            }
            res = lines.next_line() => {
                let line = match res? {
                    Some(l) => l,
                    None => break,
                };
                if line.trim().is_empty() {
                    continue;
                }
                let frame: RpcStreamFrame = serde_json::from_str(&line)?;
                match frame {
                    RpcStreamFrame::Progress {
                        phase,
                        message,
                        percent,
                    } => {
                        if show_progress {
                            if phase == "Retrieving image" {
                                if let Some(pct) = percent {
                                    retrieving_progress = Some((pct, message.clone()));
                                }
                            } else {
                                retrieving_progress = None;
                                if let Some(pct) = percent {
                                    current_status = format!("[{:>3}%] {}", pct, message);
                                } else {
                                    current_status = message.clone();
                                }
                            }
                            if !is_terminal {
                                if let Some(pct) = percent {
                                    eprintln!("[{:>3}%] {}", pct, message);
                                } else {
                                    eprintln!("{}", message);
                                }
                                let _ = io::stderr().flush();
                            }
                        }
                    }
                    RpcStreamFrame::Response(resp) => {
                        if last_was_inline {
                            eprint!("\r\x1b[2K");
                            let _ = io::stderr().flush();
                        }
                        match *resp {
                            RpcResponse::Error(err_msg) => bail!("{}", err_msg),
                            other => return Ok(other),
                        }
                    }
                }
            }
        }
    }

    bail!("Daemon closed connection before sending a response")
}

fn parse_id_mappings(mappings: &[String]) -> Result<Vec<(i32, i32)>> {
    let mut out = Vec::with_capacity(mappings.len());
    for m in mappings {
        let (h, g) = m.split_once(':').with_context(|| {
            format!("Invalid ID mapping '{}': expected <host_id>:<guest_id>", m)
        })?;
        out.push((h.parse()?, g.parse()?));
    }
    Ok(out)
}

/// Splits `<local-path>:<instance-path>` while correctly handling Windows drive letters (`C:\foo:/mnt/foo`).
fn split_host_guest_mount(spec: &str) -> Option<(String, String)> {
    // Find the last ':' that separates host path from guest path
    let idx = spec.rfind(':')?;
    if idx <= 1 {
        return None;
    }
    let host = spec[..idx].to_string();
    let guest = spec[idx + 1..].to_string();
    Some((host, guest))
}
