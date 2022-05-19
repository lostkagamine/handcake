pub mod midi;
pub mod gamepad;

pub trait ApiProvider {
    type Arguments;

    fn register_api(l: &mlua::Lua, args: Self::Arguments) -> anyhow::Result<()>;
}