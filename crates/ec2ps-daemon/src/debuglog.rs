//! User-data debugging support: cloud-init status parsing, serial-console tailing,
//! post-launch result reports, and host-side debug bundles.

use anyhow::{Context, Result};
use ec2ps_core::ssh::SshManager;
use ec2ps_core::state::InstanceRecord;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Separator printed between sections of a multi-file SSH dump.
const SECTION_MARKER: &str = "@@EC2PASS-SECTION@@";

/// Guest-side trace log written when an instance is launched with `--trace`.
pub const TRACE_LOG_PATH: &str = "/var/log/ec2pass/userdata-trace.log";

/// Lines of `cloud-init-output.log` shown in the launch report when user-data failed.
const REPORT_TAIL_LINES: usize = 20;

/// Extracts the value of the `status:` line from `cloud-init status [--long]` output.
pub fn parse_cloud_init_status(output: &str) -> Option<String> {
    output
        .lines()
        .find_map(|l| l.trim().strip_prefix("status:"))
        .map(|s| s.trim().to_string())
}

/// True once cloud-init has finished all boot stages (successfully or not).
pub fn is_final_status(status: &str) -> bool {
    matches!(status, "done" | "error" | "degraded" | "disabled") || status.starts_with("degraded")
}

/// Incrementally reads new complete lines from the host-side serial console log.
/// Handles the file being truncated when QEMU restarts.
pub struct ConsoleTail {
    path: PathBuf,
    offset: u64,
    partial: String,
}

impl ConsoleTail {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            offset: 0,
            partial: String::new(),
        }
    }

    /// Returns complete lines appended since the last call.
    pub fn read_new_lines(&mut self) -> Vec<String> {
        let Ok(mut f) = std::fs::File::open(&self.path) else {
            return Vec::new();
        };
        let len = f.metadata().map(|m| m.len()).unwrap_or(0);
        if len < self.offset {
            self.offset = 0;
            self.partial.clear();
        }
        if len == self.offset || f.seek(SeekFrom::Start(self.offset)).is_err() {
            return Vec::new();
        }
        let mut buf = Vec::new();
        if f.read_to_end(&mut buf).is_err() {
            return Vec::new();
        }
        self.offset += buf.len() as u64;
        self.partial.push_str(&String::from_utf8_lossy(&buf));

        let mut lines = Vec::new();
        while let Some(pos) = self.partial.find('\n') {
            let line: String = self.partial.drain(..=pos).collect();
            lines.push(clean_console_line(&line));
        }
        lines
    }

    /// Returns any trailing text that was not newline-terminated.
    pub fn flush_partial(&mut self) -> Option<String> {
        if self.partial.trim().is_empty() {
            self.partial.clear();
            None
        } else {
            Some(clean_console_line(&std::mem::take(&mut self.partial)))
        }
    }
}

/// Strips CR and ANSI escape sequences that serial consoles emit.
fn clean_console_line(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\u{1b}' => {
                if chars.peek() == Some(&'[') {
                    chars.next();
                    for n in chars.by_ref() {
                        if n.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
            }
            '\r' | '\n' => {}
            _ => out.push(c),
        }
    }
    out
}

/// Incrementally reads complete new lines of the guest's `/var/log/cloud-init-output.log`
/// over SSH (cloud-init on AL2023 does not echo user-data output to the serial console).
#[derive(Default)]
pub struct GuestLogTail {
    lines_seen: usize,
}

impl GuestLogTail {
    pub async fn poll(
        &mut self,
        ssh: &SshManager,
        name: &str,
        instance_dir: &Path,
        ssh_port: u16,
        ssh_user: &str,
    ) -> Vec<String> {
        let cmd = format!(
            "sudo tail -n +{} /var/log/cloud-init-output.log 2>/dev/null",
            self.lines_seen + 1
        );
        let Ok((out, _, _)) = ssh
            .exec_command(name, instance_dir, ssh_port, ssh_user, &[cmd], None)
            .await
        else {
            return Vec::new();
        };
        let mut lines: Vec<String> = out.split('\n').map(str::to_string).collect();
        // The last element is either "" (output ended with a newline) or a line still being
        // written; drop it so it is re-read complete next time.
        lines.pop();
        self.lines_seen += lines.len();
        lines
    }
}

async fn ssh_capture(ssh: &SshManager, rec: &InstanceRecord, script: &str) -> Result<String> {
    let (stdout, _stderr, _code) = ssh
        .exec_command(
            &rec.details.name,
            &rec.instance_dir,
            rec.details.ssh_port,
            &rec.details.ssh_user,
            &[script.to_string()],
            None,
        )
        .await?;
    Ok(stdout)
}

/// Builds the post-launch report. The VM keeps running regardless of the outcome; a failing
/// user script is reported as a warning, not a launch failure.
pub async fn build_launch_report(
    ssh: &SshManager,
    rec: &InstanceRecord,
    include_output_tail: bool,
) -> Vec<String> {
    let script = format!(
        "cloud-init status --long 2>&1; echo '{m}'; \
         sudo grep -hA2 'Unexpected error while running command' /var/log/cloud-init.log 2>/dev/null \
           | grep -E '^(Command|Exit code):' | tail -n 20; \
         echo '{m}'; sudo tail -n {n} /var/log/cloud-init-output.log 2>/dev/null",
        m = SECTION_MARKER,
        n = REPORT_TAIL_LINES
    );
    let out = match ssh_capture(ssh, rec, &script).await {
        Ok(o) => o,
        Err(e) => {
            return vec![format!(
                "warning: could not read cloud-init result: {:#}",
                e
            )]
        }
    };
    let mut sections = out.split(SECTION_MARKER);
    let status_text = sections.next().unwrap_or_default();
    let failures = sections.next().unwrap_or_default();
    let tail = sections.next().unwrap_or_default();
    let status = parse_cloud_init_status(status_text).unwrap_or_else(|| "unknown".to_string());

    let mut lines = Vec::new();
    if status == "done" {
        lines.push("cloud-init: done (user-data completed without errors)".to_string());
    } else {
        lines.push(format!(
            "warning: cloud-init finished with status '{}'. The instance is running; inspect the logs below.",
            status
        ));
        for l in status_text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with("status:") && !l.starts_with("time:"))
        {
            lines.push(format!("  {}", l));
        }
        let failed = summarize_failed_commands(failures);
        if !failed.is_empty() {
            lines.push("failed commands (from /var/log/cloud-init.log):".to_string());
            lines.extend(failed.into_iter().map(|l| format!("  {}", l)));
        }
        let tail: Vec<&str> = tail.lines().collect();
        if include_output_tail && !tail.is_empty() {
            lines.push(format!(
                "last {} lines of /var/log/cloud-init-output.log:",
                tail.len()
            ));
            lines.extend(tail.iter().map(|l| format!("  | {}", l)));
        }
    }
    lines
}

/// Pairs `Command: [...]` / `Exit code: N` lines from cloud-init's subp errors into
/// `"<command> exited with code N"`.
fn summarize_failed_commands(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cmd: Option<String> = None;
    for line in text.lines().map(str::trim) {
        if let Some(c) = line.strip_prefix("Command:") {
            let c = c.trim().trim_start_matches('[').trim_end_matches(']');
            cmd = Some(c.replace('\'', ""));
        } else if let Some(code) = line.strip_prefix("Exit code:") {
            out.push(format!(
                "{} exited with code {}",
                cmd.take()
                    .unwrap_or_else(|| "(unknown command)".to_string()),
                code.trim()
            ));
        }
    }
    out
}

/// Guest files gathered into a debug bundle: (file name in bundle, shell command).
const BUNDLE_ITEMS: &[(&str, &str)] = &[
    ("cloud-init-status.txt", "cloud-init status --long 2>&1"),
    ("cloud-init-output.log", "sudo cat /var/log/cloud-init-output.log"),
    ("cloud-init.log", "sudo cat /var/log/cloud-init.log"),
    ("result.json", "sudo cat /run/cloud-init/result.json"),
    ("user-data.txt", "sudo cat /var/lib/cloud/instance/user-data.txt"),
    ("vendor-data.txt", "sudo cat /var/lib/cloud/instance/vendor-data.txt"),
    (
        "user-scripts.txt",
        "sudo sh -c 'for f in /var/lib/cloud/instance/scripts/*; do [ -f \"$f\" ] && { echo \"### $f\"; cat \"$f\"; echo; }; done'",
    ),
    ("analyze-blame.txt", "sudo cloud-init analyze blame 2>&1 | head -n 60"),
    (
        "cloud-final.journal.txt",
        "sudo journalctl -b -u cloud-final --no-pager 2>&1",
    ),
    ("userdata-trace.log", "sudo cat /var/log/ec2pass/userdata-trace.log"),
];

/// Root directory for an instance's debug bundles (kept after the instance is purged).
pub fn bundle_root(data_dir: &Path, instance: &str) -> PathBuf {
    data_dir.join("logs").join(instance)
}

/// Collects guest logs plus the host serial console into
/// `<data_dir>/logs/<instance>/<UTC timestamp>/` using a single SSH round-trip.
pub async fn collect_bundle(
    ssh: &SshManager,
    rec: &InstanceRecord,
    data_dir: &Path,
) -> Result<PathBuf> {
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let dir = bundle_root(data_dir, &rec.details.name).join(ts);
    std::fs::create_dir_all(&dir).with_context(|| format!("create {:?}", dir))?;

    let script: String = BUNDLE_ITEMS
        .iter()
        .map(|(name, cmd)| format!("echo '{} {}'; {} 2>/dev/null; ", SECTION_MARKER, name, cmd))
        .collect();
    let out = ssh_capture(ssh, rec, &script).await?;

    for chunk in out.split(SECTION_MARKER).skip(1) {
        let (name, body) = chunk.split_once('\n').unwrap_or((chunk, ""));
        let name = name.trim();
        if BUNDLE_ITEMS.iter().any(|(n, _)| *n == name) && !body.trim().is_empty() {
            std::fs::write(dir.join(name), body)?;
        }
    }

    let console = rec.instance_dir.join("console.log");
    if console.exists() {
        let _ = std::fs::copy(&console, dir.join("console.log"));
    }
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status() {
        assert_eq!(
            parse_cloud_init_status("status: error\ntime: x\ndetail:\n..."),
            Some("error".to_string())
        );
        assert_eq!(
            parse_cloud_init_status("\nstatus: running\n"),
            Some("running".to_string())
        );
        assert_eq!(parse_cloud_init_status("garbage"), None);
        assert!(is_final_status("done"));
        assert!(is_final_status("error"));
        assert!(is_final_status("degraded done"));
        assert!(!is_final_status("running"));
        assert!(!is_final_status("not run"));
    }

    #[test]
    fn test_summarize_failed_commands() {
        let t = "Command: ['/var/lib/cloud/instance/scripts/part-001']\nExit code: 3\n";
        assert_eq!(
            summarize_failed_commands(t),
            vec!["/var/lib/cloud/instance/scripts/part-001 exited with code 3"]
        );
        assert!(summarize_failed_commands("").is_empty());
    }

    #[test]
    fn test_console_tail_incremental_and_truncate() {
        let p = std::env::temp_dir().join(format!("ec2ps_console_{}.log", rand::random::<u32>()));
        std::fs::write(&p, "line1\r\nli").unwrap();
        let mut t = ConsoleTail::new(p.clone());
        assert_eq!(t.read_new_lines(), vec!["line1"]);
        std::fs::write(&p, "line1\r\nline2\n\x1b[32mOK\x1b[0m\n").unwrap();
        assert_eq!(t.read_new_lines(), vec!["line2", "OK"]);
        // QEMU restart truncates the file.
        std::fs::write(&p, "boot\n").unwrap();
        assert_eq!(t.read_new_lines(), vec!["boot"]);
        assert!(t.flush_partial().is_none());
        let _ = std::fs::remove_file(&p);
    }
}
