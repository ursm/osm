use std::collections::HashMap;

use std::io;

use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, Device, EventType, InputEvent, Key};

const VAL_UP: i32 = 0;
const VAL_DOWN: i32 = 1;

pub type KeyMap = HashMap<Key, Key>;

pub fn handle_device(source: &mut Device, keymap: KeyMap) -> io::Result<()> {
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

    let mut pending: Option<Key> = None;

    source.grab()?;

    loop {
        pending = source.fetch_events()?.try_fold(pending, |pending, ev| process_event(ev, &mut sink, &keymap, pending))?;
    }
}

fn process_event(ev: InputEvent, sink: &mut VirtualDevice, keymap: &KeyMap, pending: Option<Key>) -> io::Result<Option<Key>> {
    let (evs, pending) = translate_event(keymap, ev, pending);

    if !evs.is_empty() {
        sink.emit(&evs)?;
    }

    Ok(pending)
}

fn translate_event(keymap: &KeyMap, ev: InputEvent, pending: Option<Key>) -> (Vec<InputEvent>, Option<Key>) {
    if ev.event_type() != EventType::KEY {
        return (vec![ev], pending);
    }

    let key = Key(ev.code());

    match ev.value() {
        VAL_DOWN => match (keymap.contains_key(&key), pending) {
            (false, None) => (vec![ev], None),
            (false, Some(pending)) => (vec![key_down(pending), ev], None),
            (true, None) => (vec![], Some(key)),
            (true, Some(pending)) => (vec![key_down(pending)], Some(key)),
        },

        VAL_UP => {
            let evs = match pending {
                Some(pending) if pending == key => match keymap.get(&pending) {
                    Some(dest) => vec![key_down(*dest), key_up(*dest)],
                    _ => vec![ev],
                },
                Some(pending) => vec![key_up(pending), key_up(key)],
                _ => vec![ev],
            };

            (evs, None)
        }

        _ => (vec![ev], pending),
    }
}

fn key_down(key: Key) -> InputEvent {
    InputEvent::new(EventType::KEY, key.code(), VAL_DOWN)
}

fn key_up(key: Key) -> InputEvent {
    InputEvent::new(EventType::KEY, key.code(), VAL_UP)
}

#[cfg(test)]
mod tests {
    use evdev::{InputEvent, Key};

    use crate::{key_down, key_up, translate_event, KeyMap, VAL_DOWN, VAL_UP};

    fn translate(ev: InputEvent, pending: Option<Key>) -> (Vec<(Key, i32)>, Option<Key>) {
        let keymap = KeyMap::from_iter([(Key::KEY_LEFTALT, Key::KEY_HOME), (Key::KEY_RIGHTALT, Key::KEY_END)]);
        let (evs, pending) = translate_event(&keymap, ev, pending);
        let evs = evs.iter().map(|ev| (Key::new(ev.code()), ev.value())).collect();

        (evs, pending)
    }

    #[test]
    fn keydown() {
        let (evs, pending) = translate(key_down(Key::KEY_A), None);
        assert_eq!(evs, vec![(Key::KEY_A, VAL_DOWN)]);
        assert_eq!(pending, None);

        let (evs, pending) = translate(key_down(Key::KEY_A), Some(Key::KEY_LEFTALT));
        assert_eq!(evs, vec![(Key::KEY_LEFTALT, VAL_DOWN), (Key::KEY_A, VAL_DOWN)]);
        assert_eq!(pending, None);

        let (evs, pending) = translate(key_down(Key::KEY_LEFTALT), None);
        assert_eq!(evs, vec![]);
        assert_eq!(pending, Some(Key::KEY_LEFTALT));

        let (evs, pending) = translate(key_down(Key::KEY_LEFTALT), Some(Key::KEY_RIGHTALT));
        assert_eq!(evs, vec![(Key::KEY_RIGHTALT, VAL_DOWN)]);
        assert_eq!(pending, Some(Key::KEY_LEFTALT));
    }

    #[test]
    fn keyup() {
        let (evs, pending) = translate(key_up(Key::KEY_A), None);
        assert_eq!(evs, vec![(Key::KEY_A, VAL_UP)]);
        assert_eq!(pending, None);

        let (evs, pending) = translate(key_up(Key::KEY_A), Some(Key::KEY_LEFTALT));
        assert_eq!(evs, vec![(Key::KEY_LEFTALT, VAL_UP), (Key::KEY_A, VAL_UP)]);
        assert_eq!(pending, None);

        let (evs, pending) = translate(key_up(Key::KEY_LEFTALT), None);
        assert_eq!(evs, vec![(Key::KEY_LEFTALT, VAL_UP)]);
        assert_eq!(pending, None);

        let (evs, pending) = translate(key_up(Key::KEY_LEFTALT), Some(Key::KEY_LEFTALT));
        assert_eq!(evs, vec![(Key::KEY_HOME, VAL_DOWN), (Key::KEY_HOME, VAL_UP)]);
        assert_eq!(pending, None);

        let (evs, pending) = translate(key_up(Key::KEY_LEFTALT), Some(Key::KEY_RIGHTALT));
        assert_eq!(evs, vec![(Key::KEY_RIGHTALT, VAL_UP), (Key::KEY_LEFTALT, VAL_UP)]);
        assert_eq!(pending, None);
    }
}
