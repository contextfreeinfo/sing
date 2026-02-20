use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use macroquad::prelude::*;
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
    // The init function itself should be sync.
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

#[derive(Clone, Default)]
struct Font {
    err: Arc<Mutex<Option<String>>>,
    font: Arc<Mutex<Option<macroquad::text::Font>>>,
    ready_font: RefCell<Option<macroquad::text::Font>>,
}

impl Font {
    pub fn err(&self) -> Option<String> {
        self.err.lock().unwrap().as_ref().map(|x| x.clone())
    }

    pub fn failed(&self) -> bool {
        self.err.lock().unwrap().is_some()
    }

    pub fn with_font<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&Option<macroquad::text::Font>) -> R,
    {
        if self.ready_font.borrow().is_none() && !self.failed() {
            let font = self.font.lock().unwrap();
            *self.ready_font.borrow_mut() = font.clone();
        }
        f(&*self.ready_font.borrow())
    }
}

// impl mlua::FromLua for &Font {
//     fn from_lua(value: mlua::Value, lua: &Lua) -> Result<Self> {
//         value
//             .as_userdata()
//             .map(|ud| ud.borrow::<Self>().unwrap())
//             .ok_or(mlua::Error::RuntimeError("".to_string()))
//     }
// }

impl mlua::UserData for Font {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("error", |_, this| Ok(this.err()));
        fields.add_field_method_get("failed", |_, this| Ok(this.err.lock().unwrap().is_some()));
        fields.add_field_method_get("ready", |_, this| Ok(this.font.lock().unwrap().is_some()));
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
        methods.add_method("font", |lua, this, resource: mlua::String| {
            let path = get_safe_path(&this.path, &resource.to_str().unwrap())
                .map_err(|e| mlua::Error::RuntimeError(e))?;
            let font_handle = Font::default();
            let font_clone = font_handle.clone();
            macroquad::experimental::coroutines::start_coroutine(async move {
                let path_str = path.to_string_lossy();
                match load_ttf_font(&path_str).await {
                    Ok(font) => {
                        *font_clone.font.lock().unwrap() = Some(font);
                    }
                    Err(err) => {
                        *font_clone.err.lock().unwrap() = Some(err.to_string());
                    }
                }
            });
            Ok(lua.create_userdata(font_handle))
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
        methods.add_method(
            "text",
            |_lua,
             _this,
             (text, x, y, font, font_size, rgb): (
                mlua::String,
                f32,
                f32,
                mlua::Value,
                u16,
                u32,
            )| {
                let text = text.to_str()?;
                let text_params = TextParams {
                    font_size,
                    color: Color::from_hex(rgb),
                    ..Default::default()
                };
                if let Some(font) = font.as_userdata() {
                    let font = font.borrow::<Font>()?;
                    font.with_font(|font| {
                        let text_params = TextParams {
                            font: font.as_ref(),
                            ..text_params
                        };
                        draw_text_ex(&text, x, y, text_params)
                    });
                } else {
                    draw_text_ex(&text, x, y, text_params);
                }
                Ok(())
            },
        );
    }
}

/// Safely joins a relative resource path to a base path, ensuring it stays
/// within the base path's parent directory.
/// TODO Require resource to start with "./"?
fn get_safe_path(base_path: &str, resource: &str) -> std::result::Result<PathBuf, String> {
    let base = Path::new(base_path);
    // Get the directory we are "locked" into
    let jail_dir = if base.is_dir() {
        base.to_path_buf()
    } else {
        base.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    };
    // Prevent absolute path hijacking
    let req_path = Path::new(resource);
    if req_path.is_absolute() {
        return Err("Absolute paths are forbidden".to_string());
    }
    // Join and Canonicalize
    // Note: canonicalize() requires the file to exist on disk.
    let full_path = jail_dir.join(req_path);
    let canonical_jail = jail_dir
        .canonicalize()
        .map_err(|e| format!("Base path invalid: {}", e))?;
    let canonical_target = full_path
        .canonicalize()
        .map_err(|e| format!("Resource not found or invalid: {}", e))?;
    // Boundary Check
    if canonical_target.starts_with(&canonical_jail) {
        Ok(canonical_target)
    } else {
        Err("Directory traversal attempt detected".to_string())
    }
}
