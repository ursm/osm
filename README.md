# osm

Alternative to [xcape](https://github.com/alols/xcape) for Wayland.

## Installation

Pre-built binaries are available. See [releases](https://github.com/ursm/osm/releases).

Or install it yourself as:

```
$ GO111MODULE=on
$ go build
$ go install
```

## Usage

Since osm creates a virtual keyboard device to write key events, it must be run as root (or set the permissions of `/dev/uinput` appropriately).

```
$ sudo ./osm -device /dev/input/event42 -keymap LeftCtrl=Esc,LeftShift=Home,RightShift=End
```

Refer to the [golang-evdev source code](https://github.com/gvalkov/golang-evdev/blob/master/ecodes.go) for the key names that can be used. Case is not distinguished.

Hint: You can see the device path of the keyboard by looking at `/proc/bus/input/devices`.

## Autostart

Use udev and systemd to recognize the connected keyboards and automatically start osm.

```
# /etc/udev/rules.d/99-osm.rules
ACTION=="add", KERNEL=="event*", ENV{ID_INPUT_KEYBOARD}=="1", ENV{DEVPATH}!="/devices/virtual/input/*", TAG+="systemd", ENV{SYSTEMD_ALIAS}+="/sys/devices/virtual/input/%k", RUN+="/bin/systemctl --no-block start osm@%k.service"
```

```
# /etc/systemd/system/osm@.service
[Unit]
BindsTo=sys-devices-virtual-input-%i.device

[Service]
ExecStart=/path/to/osm -device /dev/input/%I -keymap LeftCtrl=Esc,LeftShift=Home,RightShift=End
```

```
$ systemctl daemon-reload
$ udevadm control --reload
$ udevadm trigger --action=add
```
