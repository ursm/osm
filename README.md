# One-shot modifier (osm)

A utility for Linux systems that maps a modifier key to another key when pressed alone

## Installation

Pre-built binaries are available. See [releases](https://github.com/ursm/osm/releases).

Or build manually:

```
$ cargo build
$ target/debug/osm --help
```

## Usage

```
osm 2.0.0

Keita Urashima <ursm@ursm.jp>

A utility for Linux systems that maps a modifier key to another key when pressed alone

USAGE:
    osm --device <DEVICE> --keymap <KEYMAP>...

OPTIONS:
    -d, --device <DEVICE>
            Path of keyboard device

            Example: --device /dev/input/event42

            The device path can be found with `cat /proc/bus/input/devices` or `ls -l /dev/input/by-
            id`.

    -h, --help
            Print help information

    -k, --keymap <KEYMAP>...
            Source and destination keys in the form `SRC1=DEST1 SRC2=DEST2...`

            Example: --keymap LeftShift=Home RightShift=End

            A list of available key names can be found at [^1] (prefixed by `KEY_`). It is not case-
            sensitive.

            [^1]: https://docs.rs/evdev/latest/evdev/struct.Key.html

    -V, --version
            Print version information
```

### Notes

- Since osm creates a virtual keyboard device to emit key events, it must be run as root (or set the permissions of `/dev/uinput` appropriately).
- osm behaves strangely when a key is typed at startup. In other words, you can't start it from the shell with the enter key! To avoid this problem, use `sleep 1 && sudo osm ... ` or use the service manager as described below.

## Autostart

Use udev and systemd to recognize the connected keyboards and automatically start osm.

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
$ systemctl daemon-reload
$ udevadm control --reload
$ udevadm trigger --action=add
```
