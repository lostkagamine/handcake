use std::{sync::Arc, fs::File};
use input_linux::{
    UInputHandle,
    EventKind,
    Key,
    AbsoluteAxis,
    InputId,
    AbsoluteInfoSetup,
    AbsoluteInfo,
    InputEvent,
    KeyEvent,
    KeyState,
    EventTime,
    SynchronizeEvent,
    SynchronizeKind, AbsoluteEvent
};
use parking_lot::Mutex;
use super::ApiProvider;

fn i32_to_key(a: i32) -> Key {
    match a {
        _ if a == (Key::ButtonSouth as i32) => Key::ButtonSouth, 
        _ if a == (Key::ButtonEast as i32) => Key::ButtonEast, 
        _ if a == (Key::ButtonWest as i32) => Key::ButtonWest, 
        _ if a == (Key::ButtonNorth as i32) => Key::ButtonNorth, 
        _ if a == (Key::ButtonStart as i32) => Key::ButtonStart, 
        _ if a == (Key::ButtonSelect as i32) => Key::ButtonSelect, 
        _ if a == (Key::ButtonMode as i32) => Key::ButtonMode, 
        _ if a == (Key::ButtonTL as i32) => Key::ButtonTL, 
        _ if a == (Key::ButtonTR as i32) => Key::ButtonTR, 
        _ if a == (Key::ButtonThumbl as i32) => Key::ButtonThumbl, 
        _ if a == (Key::ButtonThumbr as i32) => Key::ButtonThumbr,
        _ => Key::Unknown
    }
}

fn i32_to_absaxis(a: i32) -> AbsoluteAxis {
    match a {
        _ if a == (AbsoluteAxis::X as i32) => AbsoluteAxis::X,
        _ if a == (AbsoluteAxis::Y as i32) => AbsoluteAxis::Y,
        _ if a == (AbsoluteAxis::RX as i32) => AbsoluteAxis::RX,
        _ if a == (AbsoluteAxis::RY as i32) => AbsoluteAxis::RY,
        _ if a == (AbsoluteAxis::Hat2Y as i32) => AbsoluteAxis::Hat2Y,
        _ if a == (AbsoluteAxis::Hat2X as i32) => AbsoluteAxis::Hat2X,
        _ if a == (AbsoluteAxis::Hat0Y as i32) => AbsoluteAxis::Hat0Y,
        _ if a == (AbsoluteAxis::Hat0X as i32) => AbsoluteAxis::Hat0X,
        _ => AbsoluteAxis::Reserved,
    }
}

pub struct Gamepad;
impl ApiProvider for Gamepad {
    type Arguments = (UInputHandle<File>,);

    fn register_api(l: &mlua::Lua, args: Self::Arguments) -> anyhow::Result<()> {
        let (uinput,) = args;
        let outest = Arc::new(Mutex::new(uinput));

        let tab = l.create_table()?;

        // Xbox names
        tab.set("BTN_A", Key::ButtonSouth as i32)?;
        tab.set("BTN_B", Key::ButtonEast as i32)?;
        tab.set("BTN_X", Key::ButtonWest as i32)?;
        tab.set("BTN_Y", Key::ButtonNorth as i32)?;
        tab.set("BTN_MENU", Key::ButtonStart as i32)?;
        tab.set("BTN_START", Key::ButtonStart as i32)?;
        tab.set("BTN_VIEW", Key::ButtonSelect as i32)?;
        tab.set("BTN_SELECT", Key::ButtonSelect as i32)?;
        tab.set("BTN_MODE", Key::ButtonMode as i32)?;
        tab.set("BTN_XBOX", Key::ButtonMode as i32)?;
        tab.set("BTN_LB", Key::ButtonTL as i32)?;
        tab.set("BTN_RB", Key::ButtonTR as i32)?;
        tab.set("BTN_LT", Key::ButtonTL2 as i32)?;
        tab.set("BTN_RT", Key::ButtonTR2 as i32)?;
        tab.set("BTN_LS", Key::ButtonThumbl as i32)?;
        tab.set("BTN_RS", Key::ButtonThumbr as i32)?;

        // PlayStation names
        tab.set("BTN_CROSS", Key::ButtonSouth as i32)?;
        tab.set("BTN_CIRCLE", Key::ButtonEast as i32)?;
        tab.set("BTN_SQUARE", Key::ButtonWest as i32)?;
        tab.set("BTN_TRIANGLE", Key::ButtonNorth as i32)?;
        tab.set("BTN_OPTIONS", Key::ButtonStart as i32)?;
        tab.set("BTN_SHARE", Key::ButtonSelect as i32)?;
        tab.set("BTN_PS", Key::ButtonMode as i32)?;
        tab.set("BTN_L1", Key::ButtonTL as i32)?;
        tab.set("BTN_R1", Key::ButtonTR as i32)?;
        tab.set("BTN_L2", Key::ButtonTL2 as i32)?;
        tab.set("BTN_R2", Key::ButtonTR2 as i32)?;
        tab.set("BTN_L3", Key::ButtonThumbl as i32)?;
        tab.set("BTN_R3", Key::ButtonThumbr as i32)?;

        // Generic names
        tab.set("BTN_SOUTH", Key::ButtonSouth as i32)?;
        tab.set("BTN_EAST", Key::ButtonEast as i32)?;
        tab.set("BTN_WEST", Key::ButtonWest as i32)?;
        tab.set("BTN_NORTH", Key::ButtonNorth as i32)?;
        tab.set("BTN_LBUMPER", Key::ButtonTL as i32)?;
        tab.set("BTN_RBUMPER", Key::ButtonTR as i32)?;
        tab.set("BTN_LTRIGGER", Key::ButtonTL2 as i32)?;
        tab.set("BTN_RTRIGGER", Key::ButtonTR2 as i32)?;
        tab.set("BTN_LTHUMB", Key::ButtonThumbl as i32)?;
        tab.set("BTN_RTHUMB", Key::ButtonThumbr as i32)?;

        tab.set("AXIS_LSTICK_X", AbsoluteAxis::X as i32)?;
        tab.set("AXIS_LSTICK_Y", AbsoluteAxis::Y as i32)?;
        tab.set("AXIS_RSTICK_X", AbsoluteAxis::RX as i32)?;
        tab.set("AXIS_RSTICK_Y", AbsoluteAxis::RY as i32)?;
        tab.set("AXIS_LTRIGGER", AbsoluteAxis::Hat2Y as i32)?;
        tab.set("AXIS_RTRIGGER", AbsoluteAxis::Hat2X as i32)?;
        tab.set("AXIS_DPAD_X", AbsoluteAxis::Hat0X as i32)?;
        tab.set("AXIS_DPAD_Y", AbsoluteAxis::Hat0Y as i32)?;

        {
            let outer = outest.clone();
            tab.set("create", l.create_function(move |l, (id,): (Option<String>,)| {
                let uinput = outer.lock();

                // https://docs.kernel.org/input/gamepad.html

                // Buttons
                uinput.set_evbit(EventKind::Key)?;
                uinput.set_keybit(Key::ButtonSouth)?; // A
                uinput.set_keybit(Key::ButtonEast)?; // B
                uinput.set_keybit(Key::ButtonNorth)?; // Y
                uinput.set_keybit(Key::ButtonWest)?; // X

                uinput.set_keybit(Key::ButtonStart)?; // Start/Menu
                uinput.set_keybit(Key::ButtonSelect)?; // Select/View

                uinput.set_keybit(Key::ButtonTL)?; // LB
                uinput.set_keybit(Key::ButtonTR)?; // RB
                uinput.set_keybit(Key::ButtonTL2)?; // LT
                uinput.set_keybit(Key::ButtonTR2)?; // RT

                uinput.set_keybit(Key::ButtonThumbl)?; // LS button (left stick click)
                uinput.set_keybit(Key::ButtonThumbr)?; // RS button (right stick click)

                uinput.set_keybit(Key::ButtonMode)?; // Xbox button
                
                // Axes
                uinput.set_evbit(EventKind::Absolute)?;
                uinput.set_absbit(AbsoluteAxis::X)?; // LS X
                uinput.set_absbit(AbsoluteAxis::Y)?; // LS Y
                uinput.set_absbit(AbsoluteAxis::RX)?; // RS X
                uinput.set_absbit(AbsoluteAxis::RY)?; // RS Y

                uinput.set_absbit(AbsoluteAxis::Hat2Y)?; // Left trigger (analogue)
                uinput.set_absbit(AbsoluteAxis::Hat2X)?; // Right trigger (analogue)

                uinput.set_absbit(AbsoluteAxis::Hat0X)?; // D-pad left/right (-/+)
                uinput.set_absbit(AbsoluteAxis::Hat0Y)?; // D-pad up/down (-/+)

                let mut vendor = 0x045e; // Microsoft Corp.
                let mut product = 0x0b12; // Xbox Wireless Controller
                if let Some(id) = id {
                    let a = id.split(":").collect::<Vec<&str>>();
                    let (ven, prd) = (a[0usize], a[1usize]);
                    vendor = u16::from_str_radix(ven, 16).unwrap();
                    product = u16::from_str_radix(prd, 16).unwrap();
                }

                // Create the uinput device
                let input_id = InputId {
                    bustype: input_linux::sys::BUS_USB,
                    vendor,
                    product,
                    version: 0,
                };
                let device_name = b"handcake Virtual Controller";

                const JOYSTICK: AbsoluteInfo = AbsoluteInfo {
                    flat: 0, // Deadzone
                    value: 0,
                    minimum: -32767,
                    maximum: 32767,
                    fuzz: 0,
                    resolution: 10,
                };

                const TRIGGER: AbsoluteInfo = AbsoluteInfo {
                    flat: 0,
                    ..JOYSTICK
                };

                uinput.create(&input_id, device_name, 0, &[
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::X,
                        info: JOYSTICK,
                    },
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::Y,
                        info: JOYSTICK,
                    },
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::RX,
                        info: JOYSTICK,
                    },
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::RY,
                        info: JOYSTICK,
                    },
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::Hat2Y,
                        info: TRIGGER,
                    },
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::Hat2X,
                        info: TRIGGER,
                    },
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::Hat0X,
                        info: JOYSTICK,
                    },
                    AbsoluteInfoSetup {
                        axis: AbsoluteAxis::Hat0Y,
                        info: JOYSTICK,
                    },
                ])?;

                let tab = l.create_table()?;

                {
                    let uinput = outest.clone();
                    tab.set("button", l.create_function(move |_l, (key, state): (i32, bool)| {
                        let ui = uinput.lock();
                        const ZERO: EventTime = EventTime::new(0, 0);
                        let event = [
                            *InputEvent::from(KeyEvent::new(ZERO, i32_to_key(key), match state {
                                true => KeyState::PRESSED,
                                false => KeyState::RELEASED
                            })).as_raw(),
                            *InputEvent::from(SynchronizeEvent::new(ZERO, SynchronizeKind::Report, 0)).as_raw(),
                        ];
                        ui.write(&event)?;
    
                        Ok(())
                    })?)?;
                }

                {
                    let uinput = outest.clone();
                    tab.set("axis", l.create_function(move |_l, (axis, value): (i32, f32)| {
                        let ui = uinput.lock();
                        const ZERO: EventTime = EventTime::new(0, 0);
                        let axis_value: i32 = (32768.0 * value).round() as i32;
                        let event = [
                            *InputEvent::from(AbsoluteEvent::new(ZERO, i32_to_absaxis(axis), axis_value)).as_raw(),
                            *InputEvent::from(SynchronizeEvent::new(ZERO, SynchronizeKind::Report, 0)).as_raw(),
                        ];
                        ui.write(&event)?;

                        Ok(())
                    })?)?;
                }
    
                Ok(tab)
            })?)?;
        }


        l.globals().set("gamepad", tab)?;

        Ok(())
    }
}