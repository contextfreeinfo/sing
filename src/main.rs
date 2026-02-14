use std::fs;

use macroquad::prelude::*;
use mlua::{Lua, Result, StdLib};

#[derive(argh::FromArgs)]
/// Run a Sing program.
struct Args {
    #[argh(positional)]
    script: String,
}

fn main() -> Result<()> {
    let args = argh::from_env::<Args>();

    let lua = Lua::new_with(
        // TODO Disable globals also!!!
        StdLib::BIT | StdLib::COROUTINE | StdLib::MATH | StdLib::STRING | StdLib::TABLE,
        mlua::LuaOptions::default(),
    )?;
    lua.sandbox(true)?;
    let sys = lua.create_table()?;
    lua.globals().set("sys", sys)?;

    let script = fs::read_to_string(&args.script)?;
    let script: mlua::Table = lua.load(script).eval()?;

    macroquad::Window::from_config(
        Conf {
            window_title: "Sing".into(),
            fullscreen: true,
            ..Default::default()
        },
        async move {
            // TODO Better error handling.
            run_loop(script).await.unwrap();
        },
    );
    Ok(())
}

async fn run_loop(script: mlua::Table) -> Result<()> {
    let draw: mlua::Function = script.get("draw")?;
    let mut pos = Vec2::ZERO;
    let size = Vec2::new(40.0, 40.0);
    let mut dir = Vec2::new(1.0, 1.0);
    let speed = 400.0;
    let surf = Surface;
    loop {
        let screen_size = Vec2::new(screen_width(), screen_height());
        draw.call::<()>(surf)?;
        draw_rectangle(pos.x, pos.y, size.x, size.y, WHITE);
        let end = pos + size;
        if pos.x < 0.0 {
            dir.x = 1.0;
        } else if end.x > screen_size.x {
            dir.x = -1.0;
        }
        if pos.y < 0.0 {
            dir.y = 1.0;
        } else if end.y > screen_size.y {
            dir.y = -1.0;
        }
        pos += dir * speed * get_frame_time();
        draw_text(&format!("FPS: {}", get_fps()), 200.0, 150.0, 80.0, YELLOW);
        next_frame().await
    }
}

#[derive(Clone, Copy)]
struct Surface;

impl mlua::UserData for Surface {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("clear", |_lua, _this, rgb: u32| {
            clear_background(Color::from_hex(rgb));
            Ok(())
        });
    }
}
