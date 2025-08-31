use std::collections::HashMap;

use std::io;

use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, Device, EventType, InputEvent, KeyCode, KeyEvent};

const VAL_UP: i32 = 0;
const VAL_DOWN: i32 = 1;

pub type KeyMap = HashMap<KeyCode, KeyCode>;

pub fn handle_device(source: &mut Device, keymap: KeyMap) -> io::Result<()> {
    let mut keys = AttributeSet::<KeyCode>::new();

    if let Some(supported) = source.supported_keys() {
        for k in supported.iter() {
            keys.insert(k);
        }
    }

    for k in keymap.values() {
        keys.insert(*k);
    }

    let mut sink = VirtualDevice::builder()?
        .name(&format!("osm Virtual Keyboard (source: {})", source.name().unwrap_or("Unnamed Device")))
        .with_keys(&keys)?
        .build()?;

    let mut pending: Option<KeyCode> = None;

    source.grab()?;

    loop {
        pending = source.fetch_events()?.try_fold(pending, |pending, ev| process_event(ev, &mut sink, &keymap, pending))?;
    }
}

fn process_event(ev: InputEvent, sink: &mut VirtualDevice, keymap: &KeyMap, pending: Option<KeyCode>) -> io::Result<Option<KeyCode>> {
    let (evs, pending) = translate_event(keymap, ev, pending);

    if !evs.is_empty() {
        sink.emit(&evs)?;
    }

    Ok(pending)
}

fn translate_event(keymap: &KeyMap, ev: InputEvent, pending: Option<KeyCode>) -> (Vec<InputEvent>, Option<KeyCode>) {
    if ev.event_type() != EventType::KEY {
        return (vec![ev], pending);
    }

    let key = KeyCode(ev.code());

    match ev.value() {
        VAL_DOWN => match (keymap.contains_key(&key), pending) {
            (false, None) => (vec![ev], None),
            (false, Some(pending)) => (vec![key_down(pending), ev], None),
            (true, None) => (vec![], Some(key)),
            (true, Some(pending)) => (vec![key_down(pending)], Some(key)),
        },

        VAL_UP => {
            let evs = match pending.map(|pending| {
                let dest = if pending == key { keymap.get(&pending) } else { None };

                (pending, dest)
            }) {
                Some((_, Some(dest))) => vec![key_down(*dest), key_up(*dest)],
                Some((pend, _)) => vec![key_down(pend), ev],
                _ => vec![ev],
            };

            (evs, None)
        }

        _ => (vec![ev], pending),
    }
}

fn key_down(key: KeyCode) -> InputEvent {
    KeyEvent::new(key, VAL_DOWN).into()
}

fn key_up(key: KeyCode) -> InputEvent {
    KeyEvent::new(key, VAL_UP).into()
}

#[cfg(test)]
mod tests {
    use evdev::{InputEvent, KeyCode};

    use crate::{key_down, key_up, translate_event, KeyMap, VAL_DOWN, VAL_UP};

    fn translate(ev: InputEvent, pending: Option<KeyCode>) -> (Vec<(KeyCode, i32)>, Option<KeyCode>) {
        let keymap = KeyMap::from_iter([(KeyCode::KEY_LEFTALT, KeyCode::KEY_HOME), (KeyCode::KEY_RIGHTALT, KeyCode::KEY_END)]);
        let (evs, pending) = translate_event(&keymap, ev, pending);
        let evs = evs.iter().map(|ev| (KeyCode::new(ev.code()), ev.value())).collect();

        (evs, pending)
    }

    #[test]
    fn keydown() {
        let (evs, pending) = translate(key_down(KeyCode::KEY_A), None);
        assert_eq!(evs, vec![(KeyCode::KEY_A, VAL_DOWN)]);
        assert_eq!(pending, None);

        let (evs, pending) = translate(key_down(KeyCode::KEY_A), Some(KeyCode::KEY_LEFTALT));
        assert_eq!(evs, vec![(KeyCode::KEY_LEFTALT, VAL_DOWN), (KeyCode::KEY_A, VAL_DOWN)]);
        assert_eq!(pending, None);

        let (evs, pending) = translate(key_down(KeyCode::KEY_LEFTALT), None);
        assert_eq!(evs, vec![]);
        assert_eq!(pending, Some(KeyCode::KEY_LEFTALT));

        let (evs, pending) = translate(key_down(KeyCode::KEY_LEFTALT), Some(KeyCode::KEY_RIGHTALT));
        assert_eq!(evs, vec![(KeyCode::KEY_RIGHTALT, VAL_DOWN)]);
        assert_eq!(pending, Some(KeyCode::KEY_LEFTALT));
    }

    #[test]
    fn keyup() {
        let (evs, pending) = translate(key_up(KeyCode::KEY_A), None);
        assert_eq!(evs, vec![(KeyCode::KEY_A, VAL_UP)]);
        assert_eq!(pending, None);

        let (evs, pending) = translate(key_up(KeyCode::KEY_A), Some(KeyCode::KEY_LEFTALT));
        assert_eq!(evs, vec![(KeyCode::KEY_LEFTALT, VAL_DOWN), (KeyCode::KEY_A, VAL_UP)]);
        assert_eq!(pending, None);

        let (evs, pending) = translate(key_up(KeyCode::KEY_LEFTALT), None);
        assert_eq!(evs, vec![(KeyCode::KEY_LEFTALT, VAL_UP)]);
        assert_eq!(pending, None);

        let (evs, pending) = translate(key_up(KeyCode::KEY_LEFTALT), Some(KeyCode::KEY_LEFTALT));
        assert_eq!(evs, vec![(KeyCode::KEY_HOME, VAL_DOWN), (KeyCode::KEY_HOME, VAL_UP)]);
        assert_eq!(pending, None);

        let (evs, pending) = translate(key_up(KeyCode::KEY_LEFTALT), Some(KeyCode::KEY_RIGHTALT));
        assert_eq!(evs, vec![(KeyCode::KEY_RIGHTALT, VAL_DOWN), (KeyCode::KEY_LEFTALT, VAL_UP)]);
        assert_eq!(pending, None);
    }
}
