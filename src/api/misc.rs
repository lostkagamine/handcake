use std::{time::Duration, sync::Arc};

use parking_lot::Mutex;

use super::ApiProvider;

lazy_static::lazy_static! {
    static ref START_TIME: std::time::Instant = {
        std::time::Instant::now()
    };

    static ref DELTA: Arc<Mutex<std::time::Instant>> = {
        Arc::new(Mutex::new(std::time::Instant::now()))
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
            let millis: u32 = elapsed.as_millis() as u32;
            let millis: f64 = millis.into();

            Ok(millis / 1000f64)
        })?)?;

        tab.set("delta_time", l.create_function(|_l, _: ()| {
            let t = DELTA.lock().elapsed();
            let millis = t.as_millis() as u32;
            let millis: f64 = millis.into();
            *DELTA.lock() = std::time::Instant::now();

            Ok(millis / 1000f64)
        })?)?;

        l.globals().set("misc", tab)?;

        Ok(())
    }
}