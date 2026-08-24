use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{broadcast, mpsc, watch};
use tokio::task::JoinSet;
use tracing::{debug, info, warn};

use crate::clipboard::{ClipboardBackend, NativeClipboard};
use crate::config::{Config, PeerConfig, ensure_private_dir, paths};
use crate::{service, ssh, update};

use super::Daemon;

pub async fn run(config: Config) -> Result<()> {
    let clipboard = Arc::new(NativeClipboard::new(config.max_bytes)?);
    let paths = paths()?;
    let (desired_version, _) = watch::channel(update::initial_desired_version());
    let (update_hints, hint_receiver) = mpsc::unbounded_channel();
    let update_desired = desired_version.clone();
    let shutdown = async move {
        tokio::select! {
            () = shutdown_signal() => {}
            version = update::run_auto_updates(update_desired, hint_receiver) => {
                info!(%version, "automatic update installed; requesting an explicit service restart");
                if let Err(error) = service::control(service::Action::Restart).await {
                    warn!(%error, %version, "explicit service restart failed; falling back to a clean daemon exit");
                }
            }
        }
    };
    run_daemon(
        config,
        clipboard,
        paths.socket,
        shutdown,
        desired_version,
        update_hints,
    )
    .await
}

async fn run_daemon<F>(
    config: Config,
    clipboard: Arc<dyn ClipboardBackend>,
    socket: PathBuf,
    shutdown: F,
    desired_version: watch::Sender<String>,
    update_hints: mpsc::UnboundedSender<String>,
) -> Result<()>
where
    F: Future<Output = ()>,
{
    if let Some(parent) = socket.parent() {
        ensure_private_dir(parent)?;
    }
    remove_stale_socket(&socket)?;
    let listener = UnixListener::bind(&socket).with_context(|| format!("bind {}", socket.display()))?;
    set_socket_permissions(&socket)?;
    update::mark_healthy().await?;
    let daemon = Daemon::with_updates(config, clipboard, desired_version, update_hints);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let mut tasks = JoinSet::new();
    tasks.spawn(Arc::clone(&daemon).watch_clipboard(shutdown_rx.clone()));
    tasks.spawn(accept_loop(Arc::clone(&daemon), listener, shutdown_rx.clone()));
    for peer in daemon.config.peers.clone() {
        tasks.spawn(dial_loop(Arc::clone(&daemon), peer, shutdown_rx.clone()));
    }
    info!(socket = %socket.display(), backend = daemon.clipboard.name(), "daemon ready");
    shutdown.await;
    let _ = shutdown_tx.send(true);
    tokio::time::sleep(Duration::from_millis(50)).await;
    tasks.abort_all();
    while tasks.join_next().await.is_some() {}
    let _ = tokio::fs::remove_file(&socket).await;
    Ok(())
}

async fn accept_loop(daemon: Arc<Daemon>, listener: UnixListener, mut shutdown: watch::Receiver<bool>) {
    loop {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    let daemon = Arc::clone(&daemon);
                    let shutdown = shutdown.clone();
                    tokio::spawn(async move {
                        if let Err(error) = handle_local(daemon, stream, shutdown).await {
                            debug!(%error, "local socket closed");
                        }
                    });
                }
                Err(error) => {
                    warn!(%error, "local socket accept failed");
                    return;
                }
            },
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { return; }
            }
        }
    }
}

pub(super) async fn handle_local(
    daemon: Arc<Daemon>,
    stream: UnixStream,
    shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let mut buffered = BufReader::new(stream);
    let mut command = String::new();
    buffered.read_line(&mut command).await?;
    match command.trim() {
        "BRIDGE" => {
            let (mut reader, mut writer) = tokio::io::split(buffered);
            daemon
                .serve_peer(&mut reader, &mut writer, "incoming SSH", shutdown, None)
                .await
        }
        "MONITOR" => serve_monitor(daemon, buffered.into_inner(), shutdown).await,
        "STATUS" => write_response(buffered.into_inner(), &daemon.status().await).await,
        "NOTIFY_UPDATE" => write_response(buffered.into_inner(), &daemon.notify_updates().await).await,
        _ => bail!("unknown local socket command"),
    }
}

async fn write_response<T>(mut stream: UnixStream, value: &T) -> Result<()>
where
    T: serde::Serialize,
{
    stream.write_all(&serde_json::to_vec(value)?).await?;
    stream.write_all(b"\n").await?;
    Ok(())
}

async fn serve_monitor(
    daemon: Arc<Daemon>,
    mut stream: UnixStream,
    mut shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let mut events = daemon.events.subscribe();
    loop {
        tokio::select! {
            event = events.recv() => match event {
                Ok(event) => {
                    stream.write_all(&serde_json::to_vec(&event)?).await?;
                    stream.write_all(b"\n").await?;
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => return Ok(()),
            },
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { return Ok(()); }
            }
        }
    }
}

async fn dial_loop(daemon: Arc<Daemon>, peer: PeerConfig, mut shutdown: watch::Receiver<bool>) {
    let mut backoff = Duration::from_secs(1);
    loop {
        let established = AtomicBool::new(false);
        let result = async {
            let mut child = ssh::start_bridge(&peer.ssh_command)?;
            let mut writer = child.stdin.take().context("SSH bridge stdin unavailable")?;
            let mut reader = child.stdout.take().context("SSH bridge stdout unavailable")?;
            let result = Arc::clone(&daemon)
                .serve_peer(
                    &mut reader,
                    &mut writer,
                    &peer.name,
                    shutdown.clone(),
                    Some(&established),
                )
                .await;
            let _ = child.kill().await;
            result
        }
        .await;
        if established.load(Ordering::Acquire) {
            backoff = Duration::from_secs(1);
        }
        if let Err(error) = result {
            warn!(peer = %peer.name, %error, "peer connection failed");
        }
        tokio::select! {
            () = tokio::time::sleep(backoff) => {
                backoff = (backoff * 2).min(Duration::from_secs(10));
            }
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { return; }
            }
        }
    }
}

pub(super) fn remove_stale_socket(path: &Path) -> Result<()> {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return Ok(());
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if !metadata.file_type().is_socket() {
            bail!("refusing to replace non-socket path {}", path.display());
        }
    }
    std::fs::remove_file(path)?;
    Ok(())
}

#[cfg(unix)]
fn set_socket_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_socket_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}
