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
    SynchronizeKind
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

pub struct Gamepad;
impl ApiProvider for Gamepad {
    type Arguments = (UInputHandle<File>,);

    fn register_api(l: &mlua::Lua, args: Self::Arguments) -> anyhow::Result<()> {
        let (uinput,) = args;
        let outest = Arc::new(Mutex::new(uinput));

        let tab = l.create_table()?;

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
        tab.set("BTN_LS", Key::ButtonThumbl as i32)?;
        tab.set("BTN_RS", Key::ButtonThumbr as i32)?;

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
            tab.set("create", l.create_function(move |l, _: ()| {
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

                // Create the uinput device
                let input_id = InputId {
                    bustype: input_linux::sys::BUS_USB,
                    vendor: 0x045e, // Microsoft Corporation
                    product: 0x0b12, // Xbox Wireless Controller
                    version: 0,
                };
                let device_name = b"handcake Virtual Xbox Controller";

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
    
                Ok(tab)
            })?)?;
        }


        l.globals().set("gamepad", tab)?;

        Ok(())
    }
}