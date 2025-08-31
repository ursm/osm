# One-shot modifier (osm)

A utility for Linux systems that remaps a modifier key to another key when pressed alone.

## Installation

Pre-built binaries are available. See [releases](https://github.com/ursm/osm/releases).

To build manually:

```
$ cargo build
$ target/debug/osm --help
```

## Usage

```
A utility for Linux systems that remaps a modifier key to another key when pressed alone.

Usage: osm --device <DEVICE> --keymap <KEYMAP>...

Options:
  -d, --device <DEVICE>
          Path to the keyboard device

          Example: --device /dev/input/event42

          The device path can be found with `cat /proc/bus/input/devices` or `ls -l /dev/input/by-id`.

  -k, --keymap <KEYMAP>...
          Source and destination keys in the form `SRC1=DEST1 SRC2=DEST2...`

          Example: --keymap LeftShift=Home RightShift=End

          A list of available key names can be found at [^1] (prefixed by `KEY_`). Key names are not not case-sensitive.

          [^1]: https://docs.rs/evdev/latest/evdev/struct.KeyCode.html

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Notes

- Since osm creates a virtual keyboard device to emit key events, it must be run as root (or with appropriate permissions on `/dev/uinput`).
- osm may behave unexpectedly if a key is pressed while it is starting. In particular, you cannot start it directly from a shell using the Enter key. To avoid this, run it with a delay (`sleep 1 && sudo osm ...`) or configure it to run via a service manager as described below.

## Autostart

Use udev and systemd to detect connected keyboards and automatically start osm.

```
# /etc/udev/rules.d/99-osm.rules
ACTION=="add", KERNEL=="event*", ENV{ID_INPUT_KEYBOARD}=="1", ENV{DEVPATH}!="/devices/virtual/input/*", TAG+="systemd", ENV{SYSTEMD_ALIAS}+="/sys/devices/virtual/input/%k", ENV{SYSTEMD_WANTS}+="osm@%k.service"
```

```
# /etc/systemd/system/osm@.service
[Unit]
BindsTo=sys-devices-virtual-input-%i.device
After=sys-devices-virtual-input-%i.device

[Service]
ExecStart=/path/to/osm --device /dev/input/%I --keymap LeftShift=Home RightShift=End
```

```
$ sudo systemctl daemon-reload
$ sudo udevadm control --reload
$ sudo udevadm trigger --action=add
```
