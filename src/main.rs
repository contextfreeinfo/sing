use std::fs;
use std::path::{Path, PathBuf};

use macroquad::prelude::*;
use mlua::prelude::*;
use mlua::{AnyUserData, Lua, Result, StdLib};

#[derive(argh::FromArgs)]
/// Run a Sing program.
struct Args {
    #[argh(positional)]
    script: String,
}

fn main() -> Result<()> {
    let args = argh::from_env::<Args>();

    let lua = Lua::new_with(
        StdLib::BIT | StdLib::COROUTINE | StdLib::MATH | StdLib::STRING | StdLib::TABLE,
        mlua::LuaOptions::default(),
    )?;
    lua.sandbox(true)?;
    let sys = lua.create_table()?;
    lua.globals().set("sys", sys)?;

    let script = fs::read_to_string(&args.script)?;
    let script: mlua::Table = lua.load(script).eval()?;
    let hub = lua.create_userdata(Hub {
        path: args.script,
        ..Default::default()
    })?;

    macroquad::Window::from_config(
        Conf {
            fullscreen: true,
            platform: miniquad::conf::Platform {
                webgl_version: miniquad::conf::WebGLVersion::WebGL2,
                // blocking_event_loop: true,
                // linux_wm_class: "...",
                ..Default::default()
            },
            window_title: "Sing".into(),
            ..Default::default()
        },
        async move {
            // TODO Better error handling.
            run_loop(script, hub).await.unwrap();
        },
    );
    Ok(())
}

async fn run_loop(script: mlua::Table, hub: AnyUserData) -> Result<()> {
    let init: Option<mlua::Function> = script.get("init").ok();
    let state = init
        .map(|init| init.call::<mlua::Value>(hub.clone()))
        .transpose()?;
    let update: Option<mlua::Function> = script.get("update").ok();
    let draw: Option<mlua::Function> = script.get("draw").ok();
    let surf = Surf;
    loop {
        hub.borrow_mut::<Hub>().map(|mut hub| {
            hub.frame_time = get_frame_time();
            hub.screen_size_x = screen_width();
            hub.screen_size_y = screen_height();
        })?;
        if let Some(update) = &update {
            update.call::<()>((hub.clone(), state.clone()))?;
        }
        if let Some(draw) = &draw {
            draw.call::<()>((surf, state.clone()))?;
        }
        draw_text(&format!("FPS: {}", get_fps()), 200.0, 150.0, 80.0, YELLOW);
        next_frame().await
    }
}

#[derive(Default)]
struct Hub {
    pub frame_time: f32,
    pub path: String,
    pub screen_size_x: f32,
    pub screen_size_y: f32,
}

impl mlua::UserData for Hub {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("frame_time", |_, this| Ok(this.frame_time));
        fields.add_field_method_get("screen_size_x", |_, this| Ok(this.screen_size_x));
        fields.add_field_method_get("screen_size_y", |_, this| Ok(this.screen_size_y));
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("font", |_lua, this, resource: mlua::String| {
            // get_safe_path(&this.path, resource.to_str())
            //     .map_err(|e| mlua::Error::RuntimeError(e))?;
            Ok(())
        });
    }
}

#[derive(Clone, Copy)]
struct Surf;

impl mlua::UserData for Surf {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("clear", |_lua, _this, rgb: u32| {
            // TODO Get target from `this`.
            clear_background(Color::from_hex(rgb));
            Ok(())
        });
        methods.add_method(
            "rect",
            |_lua, _this, (x0, y0, x1, y1, rgb): (f32, f32, f32, f32, u32)| {
                let size_x = (x1 - x0).abs();
                let size_y = (y1 - y0).abs();
                let x0 = x0.min(x1);
                let y0 = y0.min(y1);
                draw_rectangle(x0, y0, size_x, size_y, Color::from_hex(rgb));
                Ok(())
            },
        );
    }
}

// /// Safely joins a relative resource path to a base path, ensuring it stays 
// /// within the base path's parent directory.
// fn get_safe_path(base_root: &str, resource_request: &str) -> Result<PathBuf> {
//     let base = Path::new(base_root);
    
//     // 1. Get the directory we are "locked" into
//     let jail_dir = if base.is_dir() {
//         base.to_path_buf()
//     } else {
//         base.parent()
//             .map(|p| p.to_path_buf())
//             .unwrap_or_else(|| PathBuf::from("."))
//     };

//     // 2. Prevent absolute path hijacking
//     let req_path = Path::new(resource_request);
//     if req_path.is_absolute() {
//         return Err("Absolute paths are forbidden".to_string());
//     }

//     // 3. Join and Canonicalize
//     // Note: canonicalize() requires the file to exist on disk.
//     let full_path = jail_dir.join(req_path);
//     let canonical_jail = jail_dir.canonicalize()
//         .map_err(|e| format!("Base path invalid: {}", e))?;
//     let canonical_target = full_path.canonicalize()
//         .map_err(|e| format!("Resource not found or invalid: {}", e))?;

//     // 4. Boundary Check
//     if canonical_target.starts_with(&canonical_jail) {
//         Ok(canonical_target)
//     } else {
//         Err("Directory traversal attempt detected".to_string())
//     }
// }
