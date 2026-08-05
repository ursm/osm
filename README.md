# One-shot modifier (osm)

A utility for Linux systems that remaps a modifier key to another key when pressed alone.

## Installation

Pre-built binaries are available. See [releases](https://github.com/ursm/osm/releases).

On Gentoo, osm is packaged in the [ursm overlay](https://github.com/ursm/portage-overlay) as `app-misc/osm`.

To build and install from source:

```
$ make
$ sudo make install
```

This installs the binary under `/usr/local/bin` along with the [autostart](#autostart) files. Override `PREFIX`, `BINDIR`, `SYSCONFDIR`, `UNITDIR`, `UDEVRULESDIR`, or `DESTDIR` to put them elsewhere; the systemd unit is generated to match. If you set `CARGO_TARGET_DIR`, pass it to the second command too — `sudo` drops it from the environment.

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

udev and systemd can detect connected keyboards and start osm automatically. `make install` and the Gentoo package put the necessary files in place; the sources live in [`dist`](dist):

| Source | Installed to | Purpose |
| --- | --- | --- |
| [`dist/udev/99-osm.rules`](dist/udev/99-osm.rules) | `/usr/local/lib/udev/rules.d` | Starts `osm@.service` for each keyboard that appears |
| [`dist/systemd/osm@.service.in`](dist/systemd/osm@.service.in) | `/usr/local/lib/systemd/system` | Runs osm against that keyboard |
| [`dist/default/osm`](dist/default/osm) | `/etc/default/osm` | Your key mappings |

The unit is a template: `make install` substitutes `@BINDIR@` and `@SYSCONFDIR@` for the paths it was given. Do the same if you install it by hand. Also note that `make install` overwrites the unit, so if you wrote one yourself before osm 2.2.0, move its `--keymap` arguments into `/etc/default/osm` first.

Set your key mappings in `/etc/default/osm`. Out of the box it maps both Shift keys:

```
KEYMAP="LeftShift=Home RightShift=End"
```

Then apply everything:

```
$ sudo systemctl daemon-reload
$ sudo udevadm control --reload
$ sudo udevadm trigger --action=add
```
