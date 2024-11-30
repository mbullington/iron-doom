// Since we don't want to use SDL2 across the board (namely: so we can have viewer/editor on Web),
// we define a subset of system abstractions here.

use keycode::{KeyMap, KeyMappingCode, KeyMappingId, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemMouseButton {
    Left,
    Middle,
    Right,
}

pub type SystemKeycode = KeyMappingCode;
pub type SystemMod = KeyModifiers;

#[allow(clippy::result_unit_err)]
pub fn parse_keymap_from_usb(scancode: u16) -> Result<KeyMap, ()> {
    // Override the non-USB keys, which are broken in this crate.
    //
    // Reference:
    // https://chromium.googlesource.com/chromium/src/+/dff16958029d9a8fb9004351f72e961ed4143e83/ui/events/keycodes/dom/keycode_converter_data.inc
    // https://github.com/dfrankland/keycode/issues/12
    let id = match scancode {
        0x0010 => KeyMappingId::UsM,
        0x0011 => KeyMappingId::UsN,
        0x0012 => KeyMappingId::UsO,
        0x0013 => KeyMappingId::UsP,
        0x0014 => KeyMappingId::UsQ,
        0x0015 => KeyMappingId::UsR,
        0x0016 => KeyMappingId::UsS,
        _ => return KeyMap::from_key_mapping(keycode::KeyMapping::Usb(scancode)),
    };

    Ok(KeyMap::from(id))
}

pub enum SystemEvent {
    KeyDown {
        keycode: SystemKeycode,
        mods: SystemMod,
    },
    KeyUp {
        keycode: SystemKeycode,
        mods: SystemMod,
    },
    Text {
        text: String,
    },
    MouseMotion {
        x: i32,
        y: i32,
        xrel: i32,
        yrel: i32,
    },
    MouseWheel {
        x: i32,
        y: i32,
    },
    MouseButtonDown {
        mouse_btn: SystemMouseButton,
    },
    MouseButtonUp {
        mouse_btn: SystemMouseButton,
    },
    SizeChanged {
        width: u32,
        height: u32,
    },
}
