use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};
use tokio::time::timeout;
use uuid::Uuid;

const SSH_OPTIONS: &[&str] = &[
    "-T",
    "-oBatchMode=yes",
    "-oPasswordAuthentication=no",
    "-oKbdInteractiveAuthentication=no",
    "-oStrictHostKeyChecking=accept-new",
    "-oConnectTimeout=10",
    "-oServerAliveInterval=15",
    "-oServerAliveCountMax=3",
    "-oClearAllForwardings=yes",
    "-oLogLevel=ERROR",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProbeResult {
    pub os: String,
    pub arch: String,
    pub home: String,
    pub hostname: String,
    pub linux_clipboard: Option<LinuxClipboard>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinuxClipboard {
    pub display_available: bool,
    pub xvfb_available: bool,
    pub managed_xvfb: bool,
    pub package_manager: Option<LinuxPackageManager>,
}

impl LinuxClipboard {
    #[must_use]
    pub const fn needs_display(&self) -> bool {
        !self.display_available && !self.managed_xvfb
    }

    #[must_use]
    pub fn install_hint(&self) -> &'static str {
        match self.package_manager {
            Some(LinuxPackageManager::Apt) => "sudo apt install xvfb",
            Some(LinuxPackageManager::Dnf) => "sudo dnf install xorg-x11-server-Xvfb",
            Some(LinuxPackageManager::Pacman) => "sudo pacman -S xorg-server-xvfb",
            None => "install the Xvfb package with your system package manager",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinuxPackageManager {
    Apt,
    Dnf,
    Pacman,
}

#[must_use]
pub fn normalize_command(raw: &str) -> String {
    let trimmed = raw.trim();
    let is_ssh_command = shell_words::split(trimmed)
        .ok()
        .and_then(|words| words.into_iter().next())
        .is_some_and(|program| Path::new(&program).file_name().and_then(|name| name.to_str()) == Some("ssh"));
    if is_ssh_command {
        trimmed.to_owned()
    } else {
        format!("ssh {trimmed}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedCommand {
    program: PathBuf,
    arguments: Vec<String>,
}

pub async fn probe(raw: &str) -> Result<ProbeResult> {
    let token = format!("SCB_{}", Uuid::new_v4().simple());
    let remote = format!(
        r#"printf '{token}\t'; uname -s; uname -m; uname -n; printf '{token}\t%s\n' "$HOME"; display=0; xvfb=0; managed=0; package=none; if [ "$(uname -s)" = Linux ]; then [ -n "${{DISPLAY:-}}${{WAYLAND_DISPLAY:-}}" ] && display=1; if [ "$display" = 0 ] && command -v systemctl >/dev/null 2>&1; then manager_env=$(systemctl --user show-environment 2>/dev/null || true); printf '%s\n' "$manager_env" | grep -Eq '^(DISPLAY|WAYLAND_DISPLAY)=.+' && display=1; fi; grep -REq '^Environment="?DISPLAY=.+' "$HOME/.config/systemd/user/ssh-clipboard.service" "$HOME/.config/systemd/user/ssh-clipboard.service.d" 2>/dev/null && display=1; command -v Xvfb >/dev/null 2>&1 && xvfb=1; grep -Eq '"headless_x11"[[:space:]]*:[[:space:]]*true' "$HOME/.config/ssh-clipboard/config.json" 2>/dev/null && managed=1; if command -v apt-get >/dev/null 2>&1; then package=apt; elif command -v dnf >/dev/null 2>&1; then package=dnf; elif command -v pacman >/dev/null 2>&1; then package=pacman; fi; fi; printf '{token}\t%s\t%s\t%s\t%s\n' "$display" "$xvfb" "$managed" "$package""#
    );
    let output = timeout(Duration::from_secs(20), command(raw, &remote)?.output())
        .await
        .context("SSH verification timed out")??;
    if !output.status.success() {
        bail!(
            "passwordless SSH verification failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    parse_probe(&String::from_utf8_lossy(&output.stdout), &token)
}

pub fn start_bridge(raw: &str) -> Result<Child> {
    let mut command = command(raw, r#"exec "$HOME/.local/bin/ssh-clipboard" bridge"#)?;
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    command.kill_on_drop(true);
    command.spawn().context("start persistent SSH bridge")
}

pub async fn upload_binary(raw: &str, binary: &Path) -> Result<()> {
    let remote = r#"set -eu; umask 077; mkdir -p "$HOME/.local/bin"; tmp="$HOME/.local/bin/.ssh-clipboard.tmp.$$"; trap 'rm -f "$tmp"' EXIT; cat > "$tmp"; chmod 755 "$tmp"; mv "$tmp" "$HOME/.local/bin/ssh-clipboard"; trap - EXIT"#;
    upload(raw, remote, &tokio::fs::read(binary).await?).await
}

pub async fn upload_config_if_missing(raw: &str, bytes: &[u8]) -> Result<()> {
    let remote = r#"set -eu; umask 077; mkdir -p "$HOME/.config/ssh-clipboard"; destination="$HOME/.config/ssh-clipboard/config.json"; if [ -e "$destination" ]; then cat >/dev/null; exit 0; fi; tmp="$HOME/.config/ssh-clipboard/.config.tmp.$$"; trap 'rm -f "$tmp"' EXIT; cat > "$tmp"; chmod 600 "$tmp"; mv "$tmp" "$destination"; trap - EXIT"#;
    upload(raw, remote, bytes).await
}

pub async fn run(raw: &str, remote: &str) -> Result<Vec<u8>> {
    let output = command(raw, remote)?.output().await?;
    if !output.status.success() {
        bail!(
            "remote command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output.stdout)
}

async fn upload(raw: &str, remote: &str, bytes: &[u8]) -> Result<()> {
    let mut command = command(raw, remote)?;
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    command.kill_on_drop(true);
    let mut child = command.spawn()?;
    child
        .stdin
        .take()
        .context("SSH upload stdin unavailable")?
        .write_all(bytes)
        .await?;
    let output = child.wait_with_output().await?;
    if !output.status.success() {
        bail!(
            "SSH upload failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn command(raw: &str, remote: &str) -> Result<Command> {
    let parsed = parse_command(raw)?;
    let mut command = Command::new(parsed.program);
    command.args(SSH_OPTIONS).args(parsed.arguments).arg(remote);
    command.stdin(Stdio::null());
    command.kill_on_drop(true);
    Ok(command)
}

fn parse_command(raw: &str) -> Result<ParsedCommand> {
    let words = shell_words::split(raw).context("parse SSH command")?;
    let Some(program) = words.first() else {
        bail!("SSH command is empty");
    };
    if Path::new(program).file_name().and_then(|name| name.to_str()) != Some("ssh") {
        bail!("peer command must begin with ssh");
    }
    let mut found_destination = false;
    let mut index = 1;
    while index < words.len() {
        let word = &words[index];
        if found_destination {
            bail!("SSH command must not include a remote command");
        }
        if word == "--" {
            index += 1;
            if index >= words.len() {
                bail!("SSH command is missing a destination");
            }
            found_destination = true;
        } else if word.starts_with('-') {
            if option_takes_value(word) && word.len() == 2 {
                index += 1;
                if index >= words.len() {
                    bail!("SSH option {word} is missing a value");
                }
            }
        } else {
            found_destination = true;
        }
        index += 1;
    }
    if !found_destination {
        bail!("SSH command is missing a destination");
    }
    Ok(ParsedCommand {
        program: PathBuf::from(program),
        arguments: words[1..].to_vec(),
    })
}

fn option_takes_value(option: &str) -> bool {
    matches!(
        option,
        "-B" | "-b"
            | "-c"
            | "-D"
            | "-E"
            | "-e"
            | "-F"
            | "-I"
            | "-i"
            | "-J"
            | "-L"
            | "-l"
            | "-m"
            | "-O"
            | "-o"
            | "-P"
            | "-p"
            | "-Q"
            | "-R"
            | "-S"
            | "-W"
            | "-w"
    )
}

fn parse_probe(output: &str, token: &str) -> Result<ProbeResult> {
    let mut lines = output.lines();
    while let Some(line) = lines.next() {
        let Some(os) = line.strip_prefix(&format!("{token}\t")) else {
            continue;
        };
        let arch = lines.next().context("probe response is missing architecture")?;
        let hostname = lines.next().context("probe response is missing hostname")?;
        let home = lines
            .next()
            .and_then(|line| line.strip_prefix(&format!("{token}\t")))
            .context("probe response is missing home directory")?;
        let capabilities = lines
            .next()
            .and_then(|line| line.strip_prefix(&format!("{token}\t")))
            .context("probe response is missing clipboard capabilities")?;
        return Ok(ProbeResult {
            os: normalize_os(os),
            arch: normalize_arch(arch),
            home: home.to_owned(),
            hostname: hostname.trim().to_owned(),
            linux_clipboard: parse_linux_clipboard(os, capabilities)?,
        });
    }
    bail!("SSH connected, but its probe response was invalid")
}

fn parse_linux_clipboard(os: &str, value: &str) -> Result<Option<LinuxClipboard>> {
    if normalize_os(os) != "linux" {
        return Ok(None);
    }
    let mut fields = value.split('\t');
    let display_available = parse_probe_flag(fields.next(), "display")?;
    let xvfb_available = parse_probe_flag(fields.next(), "Xvfb")?;
    let managed_xvfb = parse_probe_flag(fields.next(), "managed Xvfb")?;
    let package_manager = match fields.next() {
        Some("apt") => Some(LinuxPackageManager::Apt),
        Some("dnf") => Some(LinuxPackageManager::Dnf),
        Some("pacman") => Some(LinuxPackageManager::Pacman),
        Some("none") => None,
        Some(_) => bail!("probe response contains an unknown package manager"),
        None => bail!("probe response is missing package manager"),
    };
    if fields.next().is_some() {
        bail!("probe response contains unexpected clipboard capabilities");
    }
    Ok(Some(LinuxClipboard {
        display_available,
        xvfb_available,
        managed_xvfb,
        package_manager,
    }))
}

fn parse_probe_flag(value: Option<&str>, name: &str) -> Result<bool> {
    match value {
        Some("0") => Ok(false),
        Some("1") => Ok(true),
        Some(_) => bail!("probe response contains an invalid {name} flag"),
        None => bail!("probe response is missing {name} flag"),
    }
}

fn normalize_os(os: &str) -> String {
    match os.trim().to_ascii_lowercase().as_str() {
        "darwin" => "darwin".to_owned(),
        "linux" => "linux".to_owned(),
        other => other.to_owned(),
    }
}

fn normalize_arch(arch: &str) -> String {
    match arch.trim().to_ascii_lowercase().as_str() {
        "x86_64" | "amd64" => "amd64".to_owned(),
        "aarch64" | "arm64" => "arm64".to_owned(),
        other => other.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_connections_accept_only_new_host_keys() {
        assert!(SSH_OPTIONS.contains(&"-oStrictHostKeyChecking=accept-new"));
    }

    #[test]
    fn accepts_options_and_quoted_paths() {
        let parsed = parse_command(r#"ssh -i "/Users/me/My Key" -p 2222 person@example.com"#).unwrap();
        assert_eq!(parsed.program, PathBuf::from("ssh"));
        assert_eq!(parsed.arguments.last().unwrap(), "person@example.com");
    }

    #[test]
    fn normalizes_hosts_and_preserves_full_commands() {
        assert_eq!(normalize_command("macbookserver"), "ssh macbookserver");
        assert_eq!(normalize_command(" user@example.com "), "ssh user@example.com");
        assert_eq!(normalize_command("ssh -p 2222 server"), "ssh -p 2222 server");
        assert_eq!(normalize_command("/usr/bin/ssh server"), "/usr/bin/ssh server");
    }

    #[test]
    fn rejects_non_ssh_and_embedded_remote_commands() {
        assert!(parse_command("bash host").is_err());
        assert!(parse_command("ssh host uname -a").is_err());
    }

    #[test]
    fn parses_probe_with_banner_noise() {
        let output = "Welcome\nTOKEN\tDarwin\narm64\nmy-mac\nTOKEN\t/Users/me\nTOKEN\t0\t0\t0\tnone\n";
        assert_eq!(
            parse_probe(output, "TOKEN").unwrap(),
            ProbeResult {
                os: "darwin".into(),
                arch: "arm64".into(),
                home: "/Users/me".into(),
                hostname: "my-mac".into(),
                linux_clipboard: None,
            }
        );
    }

    #[test]
    fn parses_headless_linux_capabilities() {
        let output = "TOKEN\tLinux\nx86_64\nserver\nTOKEN\t/home/me\nTOKEN\t0\t1\t0\tapt\n";
        let probe = parse_probe(output, "TOKEN").unwrap();
        let clipboard = probe.linux_clipboard.unwrap();
        assert!(clipboard.needs_display());
        assert!(clipboard.xvfb_available);
        assert_eq!(clipboard.install_hint(), "sudo apt install xvfb");
    }
}
