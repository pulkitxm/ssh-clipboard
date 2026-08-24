use ratatui::Terminal;
use ratatui::backend::TestBackend;

use super::*;

fn rendered(app: &SetupApp, width: u16, height: u16) -> String {
    rendered_rows(app, width, height).concat()
}

fn rendered_rows(app: &SetupApp, width: u16, height: u16) -> Vec<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .chunks(usize::from(width))
        .map(|cells| cells.iter().map(ratatui::buffer::Cell::symbol).collect())
        .collect()
}

#[test]
fn welcome_screen_renders_key_promises() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let app = SetupApp::new(runtime.handle().clone(), Config::default());
    let output = rendered(&app, 100, 28);
    assert!(output.contains("Your clipboard, without borders"));
    assert!(output.contains("No cloud account"));
    assert!(output.contains("No conversion"));
}

#[test]
fn entry_screen_renders_verification_error_without_controls() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut app = SetupApp::new(runtime.handle().clone(), Config::default());
    app.stage = Stage::Entry;
    app.error = Some("bad\u{1b}[31m connection".into());
    let output = rendered(&app, 100, 28);
    assert!(output.contains("Couldn’t verify"));
    assert!(!output.contains('\u{1b}'));
}

#[test]
fn entry_screen_shows_a_complete_command_without_a_fake_prefix() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut app = SetupApp::new(runtime.handle().clone(), Config::default());
    app.stage = Stage::Entry;
    let output = rendered(&app, 100, 28);
    assert!(output.contains("command  ssh macbookserver"));
    assert!(!output.contains("ssh  macbookserver"));
}

#[test]
fn entry_screen_offers_discovered_tailscale_machines_as_a_checklist() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut app = SetupApp::new(runtime.handle().clone(), Config::default());
    app.stage = Stage::Entry;
    app.on_message(UiMessage::TailscaleDiscovered(vec![
        tailscale::Peer {
            hostname: "Studio Mac".into(),
            dns_name: "studio.example.ts.net".into(),
            os: "macOS".into(),
        },
        tailscale::Peer {
            hostname: "server".into(),
            dns_name: "server.example.ts.net".into(),
            os: "Linux".into(),
        },
    ]));
    app.on_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
    app.on_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));

    let output = rendered(&app, 100, 28);
    assert!(app.tailscale_choices[0].selected);
    assert_eq!(app.tailscale_cursor, 1);
    assert!(output.contains("Select online machines from your Tailscale network"));
    assert!(output.contains("[✓] Studio Mac"));
    assert!(output.contains("[ ] server"));
    assert!(output.contains("Or paste a passwordless SSH command"));
    assert!(app.help().to_string().contains("space select"));
}

#[test]
fn confirmed_actions_stay_next_to_the_verified_peer() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut app = SetupApp::new(runtime.handle().clone(), Config::default());
    app.stage = Stage::Confirmed;
    app.peers.push(VerifiedPeer {
        command: "ssh macbookserver".into(),
        probe: ProbeResult {
            os: "darwin".into(),
            arch: "arm64".into(),
            home: "/Users/me".into(),
            hostname: "MacBookPro.home.local".into(),
            linux_clipboard: None,
        },
        installation: deploy::Installation {
            version: None,
            config_exists: false,
            service_exists: false,
            running: false,
        },
        headless_x11: false,
    });
    let rows = rendered_rows(&app, 100, 40);
    let verified_row = rows.iter().position(|row| row.contains("Not installed")).unwrap();
    let actions_row = rows.iter().position(|row| row.contains("enter install")).unwrap();
    assert!(actions_row > verified_row);
    assert!(actions_row - verified_row <= 6);
    assert!(actions_row < rows.len() / 2);
}

#[test]
fn confirmed_headless_linux_offers_managed_xvfb_as_opt_in() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut app = SetupApp::new(runtime.handle().clone(), Config::default());
    app.stage = Stage::Confirmed;
    app.peers.push(VerifiedPeer::new(
        "ssh server".into(),
        ProbeResult {
            os: "linux".into(),
            arch: "amd64".into(),
            home: "/home/me".into(),
            hostname: "server".into(),
            linux_clipboard: Some(crate::ssh::LinuxClipboard {
                display_available: false,
                xvfb_available: true,
                managed_xvfb: false,
                package_manager: Some(crate::ssh::LinuxPackageManager::Apt),
            }),
        },
        deploy::Installation {
            version: None,
            config_exists: false,
            service_exists: false,
            running: false,
        },
    ));

    assert!(rendered(&app, 100, 32).contains("[ ] Managed Xvfb"));
    app.on_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
    assert!(app.peers[0].headless_x11);
    assert!(rendered(&app, 100, 32).contains("[✓] Managed Xvfb"));
}

#[test]
fn ready_screen_can_add_another_peer_without_forgetting_existing_peers() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut app = SetupApp::new(runtime.handle().clone(), Config::default());
    app.stage = Stage::Ready;
    app.input = "stale input".into();
    app.error = Some("stale error".into());
    app.peers.push(VerifiedPeer {
        command: "ssh macbookserver".into(),
        probe: ProbeResult {
            os: "darwin".into(),
            arch: "arm64".into(),
            home: "/Users/me".into(),
            hostname: "MacBookPro.home.local".into(),
            linux_clipboard: None,
        },
        installation: deploy::Installation {
            version: Some(crate::update::CURRENT_VERSION.into()),
            config_exists: true,
            service_exists: true,
            running: true,
        },
        headless_x11: false,
    });
    app.on_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
    assert_eq!(app.stage, Stage::Entry);
    assert!(app.input.is_empty());
    assert!(app.error.is_none());
    assert_eq!(app.peers.len(), 1);
    assert_eq!(app.peers[0].command, "ssh macbookserver");
    assert!(app.help().to_string().contains("enter verify / install"));
}

#[test]
fn setup_merges_peers_without_erasing_existing_connections() {
    let mut peers = vec![PeerConfig {
        name: "existing".into(),
        ssh_command: "ssh existing".into(),
    }];
    merge_peer(
        &mut peers,
        PeerConfig {
            name: "new".into(),
            ssh_command: "ssh new".into(),
        },
    );
    merge_peer(
        &mut peers,
        PeerConfig {
            name: "existing-renamed".into(),
            ssh_command: "ssh existing".into(),
        },
    );
    assert_eq!(
        peers,
        vec![
            PeerConfig {
                name: "existing-renamed".into(),
                ssh_command: "ssh existing".into(),
            },
            PeerConfig {
                name: "new".into(),
                ssh_command: "ssh new".into(),
            },
        ]
    );
}
