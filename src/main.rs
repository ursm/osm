use std::error::Error;

use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;

use evdev::{Device, Key};
use osm::{handle_device, KeyMap};

#[derive(Parser)]
#[command(version, author, about)]
struct Opts {
    /// Path of keyboard device
    ///
    /// Example: --device /dev/input/event42
    ///
    /// The device path can be found with `cat /proc/bus/input/devices` or `ls -l /dev/input/by-id`.
    #[arg(short, long, required = true)]
    device: PathBuf,

    /// Source and destination keys in the form `SRC1=DEST1 SRC2=DEST2...`
    ///
    /// Example: --keymap LeftShift=Home RightShift=End
    ///
    /// A list of available key names can be found at [^1] (prefixed by `KEY_`). It is not case-sensitive.
    ///
    /// [^1]: https://docs.rs/evdev/latest/evdev/struct.Key.html
    #[arg(short, long, required = true, num_args(1..), value_parser = parse_keymap)]
    keymap: Vec<(Key, Key)>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    let mut device = Device::open(opts.device.clone()).map_err(|e| format!("{}: {}", opts.device.to_string_lossy(), e))?;
    let keymap: KeyMap = opts.keymap.clone().into_iter().collect();

    handle_device(&mut device, keymap).map_err(|e| format!("{}: {}", opts.device.to_string_lossy(), e))?;

    Ok(())
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
