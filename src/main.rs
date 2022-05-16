use std::path::{PathBuf, Path};
use clap::Parser;
use std::os::unix::io::AsRawFd;

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
        let a = std::fs::File::open(&uinput_path)?;
        a.as_raw_fd()
    };
    let uinput = input_linux::UInputHandle::new(uinput_fd);
    debug!("uinput opened");

    let script_text = std::fs::read_to_string(&script_path)?;
    let lua = mlua::Lua::new();
    let a = lua.load(&script_text);
    let a = a.set_name(&script_path.to_string_lossy().as_bytes())?;
    debug!("Evaluating initial script");
    a.exec()?;
    debug!("Calling on_script_init()");
    let globals = lua.globals();
    let on_script_init = globals.get::<&str, mlua::Function>("on_script_init")?;
    on_script_init.call::<(), ()>(())?;

    Ok(())
}
