use std::{sync::{Arc, mpsc::Sender}};
use midi_control::MidiMessage;
use midir::{Ignore, MidiInputConnection};
use mlua::{Error::ExternalError};
use parking_lot::Mutex;
use crate::Message;

use super::ApiProvider;

#[derive(Debug)]
struct MidiError(pub String);
impl std::error::Error for MidiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}
impl std::fmt::Display for MidiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("MIDI error: {}", &self.0))
    }
}

lazy_static::lazy_static! {
    static ref MIDI_CONN: Arc<Mutex<Option<MidiInputConnection<Sender<Message>>>>> = Arc::new(Mutex::new(None));
}

pub struct Midi;
impl ApiProvider for Midi {
    fn register_api(l: &mlua::Lua) -> anyhow::Result<()> {
        let tab = l.create_table()?;


        tab.set("open", l.create_function(|_l, (portno,): (usize,)| {
            let mut midi_in = midir::MidiInput::new("handcake MIDI input").unwrap();
            midi_in.ignore(Ignore::None);
            let in_ports = midi_in.ports();
            let port = match in_ports.len() {
                0 => {
                    return Err(ExternalError(Arc::new(MidiError("No ports on system.".into()))));
                },
                1 => {
                    &in_ports[0]
                },
                _ => {
                    &in_ports.get(portno).unwrap()
                }
            };

            let name = midi_in.port_name(port).unwrap();

            {
                let (_snd, _) = crate::MESSAGE.clone();
                let _snd = _snd.lock();
                let sender = _snd.clone();
                drop(_snd);

                let conn = midi_in.connect(port, &name, |_ts, data, sender|
                {
                    sender.send(Message::Midi(MidiMessage::from(data))).unwrap();
                },
                sender).unwrap();

                *MIDI_CONN.as_ref().lock() = Some(conn);
            }


            Ok(())
        })?)?;

        l.globals().set("midi", tab)?;

        Ok(())
    }
}