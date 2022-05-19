use std::{sync::Arc, fs::File};
use input_linux::{UInputHandle, EventKind, Key, AbsoluteAxis, InputId, AbsoluteInfoSetup, AbsoluteInfo};
use parking_lot::Mutex;
use super::ApiProvider;

pub struct Gamepad;
impl ApiProvider for Gamepad {
    type Arguments = (UInputHandle<File>,);

    fn register_api(l: &mlua::Lua, args: Self::Arguments) -> anyhow::Result<()> {
        let (uinput,) = args;
        let uinput = Arc::new(Mutex::new(uinput));

        let tab = l.create_table()?;

        {
            let uinput = uinput.clone();
            tab.set("create", l.create_function(move |l, _: ()| {
                let uinput = uinput.lock();

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
    
                Ok(())
            })?)?;
        }


        l.globals().set("gamepad", tab)?;

        Ok(())
    }
}