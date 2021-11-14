use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, Device, EventType, InputEvent, Key};

#[derive(Parser)]
#[clap(version, author, about)]
struct Opts {
    /// Path of keyboard device
    ///
    /// Example: --device /dev/input/event42
    ///
    /// The device path can be found with `cat /proc/bus/input/devices` or `ls -l /dev/input/by-id`.
    #[clap(short, long, required = true, parse(from_os_str))]
    device: PathBuf,

    /// Source and destination keys in the form `SRC1=DEST1 SRC2=DEST2...`
    ///
    /// Example: --keymap LeftShift=Home RightShift=End
    ///
    /// A list of available key names can be found at [^1] (prefixed by `KEY_`). It is not case-sensitive.
    ///
    /// [^1]: https://docs.rs/evdev/latest/evdev/struct.Key.html
    #[clap(short, long, required = true, multiple_values = true, parse(try_from_str = parse_keymap))]
    keymap: Vec<(Key, Key)>,
}

fn parse_keymap(s: &str) -> Result<(Key, Key), String> {
    let keys: Vec<_> = s.splitn(2, '=').collect();

    if keys.len() != 2 {
        return Err(format!("{}: Must be in the form `SRC=DEST`", s));
    }

    keys.iter()
        .map(|key| {
            let key = format!("KEY_{}", key.trim().to_uppercase());

            Key::from_str(&key).map_err(|_| format!("{}: Unknown key name", key))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|v| (v[0], v[1]))
}

#[test]
fn test_parse_keymap() {
    assert!(parse_keymap("LeftCtrl=Esc").is_ok());
    assert_eq!(parse_keymap("foo").unwrap_err(), "foo: Must be in the form `SRC=DEST`");
    assert_eq!(parse_keymap("foo=bar").unwrap_err(), "KEY_FOO: Unknown key name");
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    let mut device = Device::open(opts.device.clone()).map_err(|e| format!("{}: {}", opts.device.to_string_lossy(), e))?;
    let keymap: HashMap<_, _> = opts.keymap.clone().into_iter().collect();

    process_events(&mut device, keymap).map_err(|e| format!("{}: {}", opts.device.to_string_lossy(), e))?;

    Ok(())
}

fn process_events(source: &mut Device, keymap: HashMap<Key, Key>) -> io::Result<()> {
    let mut keys = AttributeSet::<Key>::new();

    if let Some(supported) = source.supported_keys() {
        for k in supported.iter() {
            keys.insert(k);
        }
    }

    for k in keymap.values() {
        keys.insert(*k);
    }

    let mut sink = VirtualDeviceBuilder::new()?
        .name(&format!("osm Virtual Keyboard (source: {})", source.name().unwrap_or("Unnamed Device")))
        .with_keys(&keys)?
        .build()?;

    let mut pressed: Option<Key> = None;

    source.grab()?;

    loop {
        pressed = source.fetch_events()?.try_fold(pressed, |pressed, ev| handle_key_event(ev, &mut sink, &keymap, pressed))?;
    }
}

const KEY_UP: i32 = 0;
const KEY_DOWN: i32 = 1;

fn handle_key_event(ev: InputEvent, sink: &mut VirtualDevice, keymap: &HashMap<Key, Key>, pressed: Option<Key>) -> io::Result<Option<Key>> {
    if ev.event_type() != EventType::KEY {
        sink.emit(&[ev])?;

        return Ok(pressed);
    }

    let key = Key(ev.code());

    match ev.value() {
        KEY_DOWN => {
            if let Some(pressed) = pressed {
                sink.emit(&[key_event(pressed, KEY_DOWN)])?;
            }

            if keymap.contains_key(&key) {
                Ok(Some(key))
            } else {
                sink.emit(&[ev])?;

                Ok(None)
            }
        }

        KEY_UP => {
            if let Some(pressed) = pressed {
                if let Some(substitute) = keymap.get(&pressed) {
                    sink.emit(&[key_event(*substitute, KEY_DOWN), key_event(*substitute, KEY_UP)])?;
                } else {
                    sink.emit(&[ev])?;
                }
            } else {
                sink.emit(&[ev])?;
            }

            Ok(None)
        }

        _ => {
            sink.emit(&[ev])?;

            Ok(pressed)
        }
    }
}

fn key_event(key: Key, value: i32) -> InputEvent {
    InputEvent::new(EventType::KEY, key.code(), value)
}
