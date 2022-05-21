use std::time::Duration;

use super::ApiProvider;

lazy_static::lazy_static! {
    static ref START_TIME: std::time::Instant = {
        std::time::Instant::now()
    };
}

pub struct Misc;
impl ApiProvider for Misc {
    type Arguments = ();

    fn register_api(l: &mlua::Lua, _args: Self::Arguments) -> anyhow::Result<()> {
        let tab = l.create_table()?;

        tab.set("sleep", l.create_function(|_l, (time,): (f32,)| {
            std::thread::sleep(Duration::from_secs_f32(time));
            Ok(())
        })?)?;

        tab.set("time", l.create_function(|_l, _: ()| {
            let t = std::time::Instant::now();
            let elapsed = t.duration_since(*START_TIME);

            Ok(elapsed.as_millis() as f32 / 1000f32)
        })?)?;

        l.globals().set("misc", tab)?;

        Ok(())
    }
}