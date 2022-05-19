pub mod midi;

pub trait ApiProvider {
    fn register_api(l: &mlua::Lua) -> anyhow::Result<()>;
}