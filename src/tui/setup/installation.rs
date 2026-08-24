use std::sync::mpsc::Sender;

use anyhow::{Context, Result};

use crate::config::{Config, PeerConfig};
use crate::deploy;

use super::{UiMessage, VerifiedPeer};

pub(super) async fn install_all(
    config: Config,
    peers: Vec<VerifiedPeer>,
    sender: Sender<UiMessage>,
) -> Result<()> {
    let mut local = config;
    for peer in &peers {
        merge_peer(
            &mut local.peers,
            PeerConfig {
                name: peer.probe.hostname.clone(),
                ssh_command: peer.command.clone(),
            },
        );
    }
    local.save()?;

    for peer in &peers {
        let name = peer.probe.hostname.clone();
        let progress_sender = sender.clone();
        let outcome = deploy::install_remote(&peer.command, &peer.probe, peer.headless_x11, |_, detail| {
            let _ = progress_sender.send(UiMessage::Progress {
                peer: name.clone(),
                detail: detail.to_owned(),
                complete: false,
            });
        })
        .await
        .with_context(|| format!("install {name}"))?;
        let _ = sender.send(UiMessage::Progress {
            peer: name,
            detail: outcome.detail().into(),
            complete: true,
        });
    }

    let _ = sender.send(UiMessage::Progress {
        peer: local.node_name.clone(),
        detail: "Installing this machine’s service".into(),
        complete: false,
    });
    let outcome = deploy::install_local_service().await?;
    let _ = sender.send(UiMessage::Progress {
        peer: local.node_name,
        detail: outcome.detail().into(),
        complete: true,
    });
    Ok(())
}

pub(super) fn merge_peer(peers: &mut Vec<PeerConfig>, configured: PeerConfig) {
    if let Some(existing) = peers
        .iter_mut()
        .find(|existing| existing.ssh_command == configured.ssh_command || existing.name == configured.name)
    {
        *existing = configured;
    } else {
        peers.push(configured);
    }
}
