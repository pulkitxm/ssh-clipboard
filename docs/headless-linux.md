# Headless Linux

A headless Linux machine usually has no Wayland compositor or X11 display, so there is no native
clipboard for `ssh-clipboard` to read and write. Xvfb solves this by providing a small virtual X11
server. It is the display that owns the clipboard; it is not a clipboard-history application.

## Guided setup

Run `ssh-clipboard setup` from an existing machine and select the Linux peer. Setup checks the
peer's shell and `systemd --user` environment for a usable `DISPLAY` or `WAYLAND_DISPLAY`.

If the peer is headless:

1. Install Xvfb on the peer if setup says it is missing:

   ```sh
   # Ubuntu or Debian
   sudo apt install xvfb

   # Fedora or RHEL
   sudo dnf install xorg-x11-server-Xvfb

   # Arch Linux
   sudo pacman -S xorg-server-xvfb
   ```

2. Verify the peer again.
3. Press `x` to opt in to **Managed Xvfb**, then install.

Setup writes two per-user systemd units. `ssh-clipboard-xvfb.service` owns a private display at
`:99`, with TCP access disabled. `ssh-clipboard.service` receives `DISPLAY=:99` and starts after
that display. The choice is stored in the node configuration, so `ssh-clipboard update` preserves
it when the service is reconciled.

Setup does not run a package manager or `sudo` for you. Installing system packages is the only
administrator-controlled step; everything after it is per-user and automated.

Existing setups are preserved. A working `DISPLAY` in the user manager, the ssh-clipboard unit,
or a systemd drop-in is treated as a graphical clipboard and does not trigger the Xvfb offer.
Setup never replaces an existing `ssh-clipboard-xvfb.service` that it did not create, and it
refuses to claim `:99` when another X server already owns it. Hand-written systemd drop-ins remain
in place during upgrades.

## Manual setup

After installing the Xvfb package, the same configuration is available without the TUI:

```sh
ssh-clipboard service install --headless-x11
ssh-clipboard status
```

To move the machine back to a real graphical session later:

```sh
ssh-clipboard service install --native-display
```

This removes and stops the managed Xvfb unit, then restores normal Wayland/X11 environment
discovery.

## Keeping user services alive

Most systemd distributions keep an active user manager available for the SSH session. If your
distribution stops user services as soon as the last login ends, enable lingering once:

```sh
sudo loginctl enable-linger "$USER"
```

Check both units and their logs with:

```sh
systemctl --user status ssh-clipboard-xvfb.service ssh-clipboard.service
journalctl --user -u ssh-clipboard-xvfb.service -u ssh-clipboard.service
```

Do not set `DISPLAY=:99` on a second independently managed X server. Either let
`ssh-clipboard service install --headless-x11` own that display or point a custom service at a
different display number.
