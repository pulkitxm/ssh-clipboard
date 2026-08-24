use std::sync::mpsc::Sender;
use std::time::Duration;

use tokio::io::AsyncBufReadExt;
use tokio::sync::{mpsc, watch};

use crate::daemon;
use crate::model::MonitorEvent;

use super::{MonitorCommand, UiMessage};

pub(super) async fn monitor_feed(sender: Sender<UiMessage>, mut shutdown: watch::Receiver<bool>) {
    loop {
        match daemon::connect_monitor().await {
            Ok(mut reader) => {
                let mut line = String::new();
                loop {
                    line.clear();
                    tokio::select! {
                        result = reader.read_line(&mut line) => match result {
                            Ok(0) => break,
                            Ok(_) => match serde_json::from_str::<MonitorEvent>(&line) {
                                Ok(event) => { let _ = sender.send(UiMessage::Event(event)); }
                                Err(error) => { let _ = sender.send(UiMessage::Error(error.to_string())); }
                            },
                            Err(error) => {
                                let _ = sender.send(UiMessage::Error(error.to_string()));
                                break;
                            }
                        },
                        changed = shutdown.changed() => {
                            if changed.is_err() || *shutdown.borrow() { return; }
                        }
                    }
                }
            }
            Err(error) => {
                let _ = sender.send(UiMessage::Error(error.to_string()));
            }
        }
        tokio::select! {
            () = tokio::time::sleep(Duration::from_secs(1)) => {}
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { return; }
            }
        }
    }
}

pub(super) async fn status_feed(sender: Sender<UiMessage>, mut shutdown: watch::Receiver<bool>) {
    loop {
        match daemon::query_status().await {
            Ok(status) => {
                let _ = sender.send(UiMessage::Status(status));
            }
            Err(error) => {
                let _ = sender.send(UiMessage::Offline(error.to_string()));
            }
        }
        tokio::select! {
            () = tokio::time::sleep(Duration::from_secs(1)) => {}
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { return; }
            }
        }
    }
}

pub(super) async fn command_feed(
    sender: Sender<UiMessage>,
    mut commands: mpsc::UnboundedReceiver<MonitorCommand>,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            command = commands.recv() => match command {
                Some(MonitorCommand::NotifyUpdates) => {
                    let message = match daemon::notify_updates().await {
                        Ok(notification) => UiMessage::UpdateNotified(notification),
                        Err(error) => UiMessage::UpdateFailed(error.to_string()),
                    };
                    let _ = sender.send(message);
                }
                None => return,
            },
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    return;
                }
            }
        }
    }
}
