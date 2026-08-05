use std::error::Error;

use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;

use evdev::{Device, KeyCode};
use osm::{handle_device, KeyMap};

#[derive(Parser)]
#[command(version, author, about)]
struct Opts {
    /// Path to the keyboard device
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
    /// With no mappings osm does nothing and exits, so an empty keymap disables it.
    ///
    /// A list of available key names can be found at [^1] (prefixed by `KEY_`). Key names are not case-sensitive.
    ///
    /// [^1]: https://docs.rs/evdev/latest/evdev/struct.KeyCode.html
    #[arg(short, long, num_args(0..), value_parser = parse_keymap)]
    keymap: Vec<Option<(KeyCode, KeyCode)>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    let keymap: KeyMap = opts.keymap.into_iter().flatten().collect();

    // No mappings: nothing to remap. Exit cleanly without grabbing the keyboard
    // so an empty keymap disables osm instead of failing to start. Say so, or a
    // typo in the configuration looks exactly like a working osm doing nothing.
    if keymap.is_empty() {
        eprintln!("No key mappings given, doing nothing.");

        return Ok(());
    }

    let mut device = Device::open(opts.device.clone()).map_err(|e| format!("{}: {}", opts.device.to_string_lossy(), e))?;

    handle_device(&mut device, keymap).map_err(|e| format!("{}: {}", opts.device.to_string_lossy(), e))?;

    Ok(())
}

fn parse_keymap(s: &str) -> Result<Option<(KeyCode, KeyCode)>, String> {
    // An unset $KEYMAP arrives as one empty argument whenever it is quoted, in a
    // hand-written unit or a wrapper script. Treat that as no mapping, so
    // quoting cannot turn "disabled" into a hard failure.
    if s.trim().is_empty() {
        return Ok(None);
    }

    let keys: Vec<_> = s.splitn(2, '=').collect();

    if keys.len() != 2 {
        return Err(format!("{}: Must be in the form `SRC=DEST`", s));
    }

    keys.iter()
        .map(|key| {
            let key = format!("KEY_{}", key.trim().to_uppercase());

            KeyCode::from_str(&key).map_err(|_| format!("{}: Unknown key name", key))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|v| Some((v[0], v[1])))
}

#[test]
fn test_parse_keymap() {
    assert_eq!(parse_keymap("LeftCtrl=Esc").unwrap(), Some((KeyCode::KEY_LEFTCTRL, KeyCode::KEY_ESC)));
    assert_eq!(parse_keymap("").unwrap(), None);
    assert_eq!(parse_keymap("  ").unwrap(), None);
    assert_eq!(parse_keymap("foo").unwrap_err(), "foo: Must be in the form `SRC=DEST`");
    assert_eq!(parse_keymap("foo=bar").unwrap_err(), "KEY_FOO: Unknown key name");
}
