pub mod midi;
pub mod gamepad;
pub mod misc;

pub trait ApiProvider {
    type Arguments;

    fn register_api(l: &mlua::Lua, args: Self::Arguments) -> anyhow::Result<()>;
}