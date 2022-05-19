pub mod api;
mod util;

use std::{path::{PathBuf, Path}, os::unix::prelude::OpenOptionsExt, sync::{Arc, mpsc::{Sender, Receiver}}};
use clap::Parser;
use midi_control::MidiMessage;
use parking_lot::Mutex;
use std::os::unix::io::AsRawFd;

use crate::api::ApiProvider;

#[macro_use]
extern crate log;

#[derive(Parser)]
struct HandcakeApplication {
    #[clap(short='s',long="--script")]
    pub script: PathBuf,
}

#[cfg(not(unix))]
compile_error!("This program is only for Unix-like systems.");

macro_rules! fatal_error {
    ($t:literal) => {
        error!($t);
        std::process::exit(1);
    };

    ($t:literal, $($x:expr),*) => {
        error!($t, $($x),*);
        std::process::exit(1);
    };
}

#[derive(Debug)]
pub enum Message {
    Midi(MidiMessage),
}

lazy_static::lazy_static! {
    static ref MESSAGE: (Arc<Mutex<Sender<Message>>>, Arc<Mutex<Receiver<Message>>>) = {
        let (send, recv) = std::sync::mpsc::channel();

        (Arc::new(Mutex::new(send)), Arc::new(Mutex::new(recv)))
    };
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let cli = HandcakeApplication::parse();
    let script_path = cli.script;
    info!("handcake v{} starting - (c)2022 rin", env!("CARGO_PKG_VERSION"));
    if !script_path.exists() {
        fatal_error!("Script at path {:?} does not exist, aborting.", script_path);
    }
    info!("Running script {:?}", script_path);

    let uinput_fd = {
        let uinput_path = Path::new("/dev").join("uinput");
        if !uinput_path.exists() {
            fatal_error!("Could not find /dev/uinput. Is uinput installed?");
        }
        let a = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(uinput_path)?;
        a.as_raw_fd()
    };
    let uinput = input_linux::UInputHandle::new(uinput_fd);
    debug!("uinput opened");

    let script_text = std::fs::read_to_string(&script_path)?;
    let lua = mlua::Lua::new();
    let a = lua.load(&script_text);
    let a = a.set_name(&script_path.to_string_lossy().as_bytes())?;

    api::midi::Midi::register_api(&lua).unwrap();

    debug!("Evaluating initial script");

    a.exec()?;
    debug!("Calling on_script_init()");

    {
        let globals = &lua.globals();
        let on_script_init = globals.get::<&str, mlua::Function>("on_script_init")?;
        on_script_init.call::<(), ()>(())?;
    }

    let lua = Arc::new(Mutex::new(lua));

    debug!("Receiving messages");
    
    std::thread::spawn(move || {
        let (_, recv) = MESSAGE.clone();

        let lock = recv.lock();
        while let Ok(x) = lock.recv() {
            if let Message::Midi(midi) = x {
                let lua = lua.lock();
                let on_midi_recv = lua.globals().get::<&str, mlua::Function>("on_midi_recv");
                if let Err(_) = on_midi_recv {
                    continue;
                }
                let on_midi_recv = on_midi_recv.unwrap();
                if let MidiMessage::Invalid = midi {
                    continue;
                }

                let tab = lua.create_table().unwrap();

                match &midi {
                    MidiMessage::NoteOn(channel, key) => {
                        tab.set("event", "note_on").unwrap();
                        tab.set("channel", util::midi_channel_to_num(channel)).unwrap();
                        tab.set("key", key.key).unwrap();
                        tab.set("vel", key.value).unwrap();
                    },
                    MidiMessage::NoteOff(channel, key) => {
                        tab.set("event", "note_off").unwrap();
                        tab.set("channel", util::midi_channel_to_num(channel)).unwrap();
                        tab.set("key", key.key).unwrap();
                        tab.set("vel", key.value).unwrap();
                    },
                    MidiMessage::ControlChange(channel, cc) => {
                        tab.set("event", "control_change").unwrap();
                        tab.set("channel", util::midi_channel_to_num(channel)).unwrap();
                        tab.set("control", cc.control).unwrap();
                        tab.set("value", cc.value).unwrap();
                    },
                    MidiMessage::ProgramChange(channel, prgm) => {
                        tab.set("event", "program_change").unwrap();
                        tab.set("channel", util::midi_channel_to_num(channel)).unwrap();
                        tab.set("program", *prgm).unwrap();
                    },
                    MidiMessage::PitchBend(channel, lsb, msb) => {
                        tab.set("channel", util::midi_channel_to_num(channel)).unwrap();
                        let true_val: u16 = ((*msb as u16) << 8) | *lsb as u16;
                        tab.set("event", "pitch_bend").unwrap();
                        tab.set("value", true_val).unwrap();
                    },
                    x @ _ => {
                        debug!("Unknown MIDI message seen: {:?}", x);
                        continue;
                    },
                }

                on_midi_recv.call::<_, ()>((tab,)).unwrap();
            }
        }
    }).join().unwrap();


    Ok(())
}
