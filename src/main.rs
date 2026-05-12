use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{env, fs};

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
        StdLib::BIT
            | StdLib::COROUTINE
            | StdLib::MATH
            | StdLib::STRING
            | StdLib::TABLE
            | StdLib::UTF8,
        mlua::LuaOptions::default(),
    )?;
    lua.sandbox(true)?;
    // TODO Customize require? Needs safer??? Use it for finding resources???
    lua.create_require_function(mlua::TextRequirer::new())?;
    let sys = lua.create_table()?;
    lua.globals().set("sys", sys)?;

    let script = fs::read_to_string(&args.script)?;
    let script: mlua::Table = lua.load(script).eval()?;
    let hub = lua.create_userdata(Hub {
        audio: Audio {
            manager: kira::AudioManager::<kira::DefaultBackend>::new(
                kira::AudioManagerSettings::default(),
            )
            .map(|x| Arc::new(Mutex::new(x)))
            .ok(),
        },
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
            // window_height: 301, // TODO Can we use this to detect size ready?
            // window_width: 301,
            window_title: "Sing".into(),
            ..Default::default()
        },
        async move {
            // TODO Better error handling.
            run_loop(lua, script, hub).await.unwrap();
        },
    );
    Ok(())
}

async fn run_loop(lua: Lua, script: mlua::Table, hub: AnyUserData) -> Result<()> {
    // Burn some frames in hopes we get screen size correct.
    for _ in 0..3 {
        hub.borrow_mut::<Hub>().map(|mut hub| {
            hub.update();
            // println!("size: {} {}", hub.screen_size_x, hub.screen_size_y);
        })?;
        next_frame().await
    }
    // Now get going.
    let init: Option<mlua::Function> = script.get("init").ok();
    // The init function itself should be sync.
    // TODO Wait to call init until we see size stabilize.
    let state = init
        .map(|init| init.call::<mlua::Value>(hub.clone()))
        .transpose()?;
    let update: Option<mlua::Function> = script.get("update").ok();
    let draw: Option<mlua::Function> = script.get("draw").ok();
    let surf = Surf;
    loop {
        if is_quit_requested() {
            // Clean up lua first.
            drop(lua);
            break Ok(());
        }
        hub.borrow_mut::<Hub>().map(|mut hub| {
            hub.update();
        })?;
        if let Some(update) = &update {
            update.call::<()>((hub.clone(), state.clone()))?;
        }
        if let Some(draw) = &draw {
            draw.call::<()>((surf, state.clone()))?;
        }
        // draw_text(&format!("FPS: {}", get_fps()), 200.0, 150.0, 80.0, YELLOW);
        next_frame().await
    }
}

#[derive(Clone)]
struct Font {
    face: mlua::AnyUserData,
    face_internal: FontFace,
    size: u16,
}

impl mlua::UserData for Font {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("face", |_, this| Ok(this.face.clone()));
        fields.add_field_method_get("size", |_, this| Ok(this.size));
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("measure", |_lua, this, text: mlua::String| {
            this.face_internal.with_internal(|face| {
                let text = &text.to_str().unwrap();
                let measure = measure_text(text, face.as_ref(), this.size, 1.0);
                Ok((measure.width, measure.height, measure.offset_y))
            })
        });
    }
}

#[derive(Clone, Default)]
struct FontFace {
    err: Arc<Mutex<Option<String>>>,
    internal: Arc<Mutex<Option<macroquad::text::Font>>>,
    ready_internal: RefCell<Option<macroquad::text::Font>>,
}

impl FontFace {
    pub fn err(&self) -> Option<String> {
        self.err.lock().unwrap().as_ref().map(|x| x.clone())
    }

    pub fn failed(&self) -> bool {
        self.err.lock().unwrap().is_some()
    }

    pub fn with_internal<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&Option<macroquad::text::Font>) -> R,
    {
        if self.ready_internal.borrow().is_none() && !self.failed() {
            let font = self.internal.lock().unwrap();
            *self.ready_internal.borrow_mut() = font.clone();
        }
        f(&self.ready_internal.borrow())
    }
}

impl mlua::UserData for FontFace {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("failed", |_, this| Ok(this.err.lock().unwrap().is_some()));
        fields.add_field_method_get("error", |_, this| Ok(this.err()));
        fields.add_field_method_get("ready", |_, this| {
            Ok(this.internal.lock().unwrap().is_some())
        });
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("font", |_lua, (this, size): (mlua::AnyUserData, u16)| {
            Ok(Font {
                face: this.clone(),
                face_internal: this.borrow::<FontFace>()?.clone(),
                size,
            })
        });
    }
}

#[derive(Clone, Default)]
struct Audio {
    pub manager: Option<Arc<Mutex<kira::AudioManager>>>,
}

impl mlua::UserData for Audio {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("play", |_, this, (sound,): (mlua::Value,)| {
            if let Some(sound) = sound.as_userdata() {
                let sound = sound.borrow::<Sound>()?;
                if let Ok(sound) = &sound.result {
                    let manager = this.manager.as_ref().unwrap();
                    manager.lock().unwrap().play(sound.clone()).ok();
                }
            }
            Ok(())
        });
    }
}

#[derive(Default)]
struct Hub {
    audio: Audio,
    pub fps: f32,
    pub frame_time: f32,
    pub path: String,
    pub screen_size_x: f32,
    pub screen_size_y: f32,
    pub text: String,
}

impl Hub {
    pub fn update(&mut self) {
        self.fps = get_fps() as f32;
        self.frame_time = get_frame_time();
        self.screen_size_x = screen_width();
        self.screen_size_y = screen_height();
        self.text.clear();
        while let Some(ch) = get_char_pressed() {
            if ch >= ' ' {
                self.text.push(ch);
            }
        }
    }
}

impl mlua::UserData for Hub {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("audio", |_, this| Ok(this.audio.clone()));
        fields.add_field_method_get("fps", |_, this| Ok(this.fps));
        fields.add_field_method_get("frameTime", |_, this| Ok(this.frame_time));
        fields.add_field_method_get("screenSizeX", |_, this| Ok(this.screen_size_x));
        fields.add_field_method_get("screenSizeY", |_, this| Ok(this.screen_size_y));
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("fontFace", |lua, this, path: mlua::String| {
            let path = get_safe_path(&this.path, &path.to_str().unwrap())
                .map_err(mlua::Error::RuntimeError)?;
            let font_handle = FontFace::default();
            let font_clone = font_handle.clone();
            macroquad::experimental::coroutines::start_coroutine(async move {
                let path_str = path.to_string_lossy();
                match load_ttf_font(&path_str).await {
                    Ok(font) => {
                        *font_clone.internal.lock().unwrap() = Some(font);
                    }
                    Err(err) => {
                        *font_clone.err.lock().unwrap() = Some(err.to_string());
                    }
                }
            });
            Ok(lua.create_userdata(font_handle))
        });
        methods.add_method("keyPressed", |_, _, key: mlua::String| {
            let key = key.to_str().unwrap();
            let key_code = match key.as_ref() {
                "Backspace" => KeyCode::Backspace,
                "Delete" => KeyCode::Delete,
                "Down" => KeyCode::Down,
                "Left" => KeyCode::Left,
                "Enter" => KeyCode::Enter,
                "Right" => KeyCode::Right,
                "Tab" => KeyCode::Tab,
                "Up" => KeyCode::Up,
                _ => return Ok(false),
            };
            Ok(is_key_pressed(key_code))
        });
        methods.add_method("sound", |lua, this, path: mlua::String| {
            let path = get_safe_path(&this.path, &path.to_str().unwrap())
                .map_err(mlua::Error::RuntimeError)?;
            let result = kira::sound::static_sound::StaticSoundData::from_file(path)
                .map_err(|err| err.to_string());
            Ok(lua.create_userdata(Sound { result }))
        });
        methods.add_method("text", |_, this, ()| Ok(this.text.clone()));
    }
}

#[derive(Clone)]
struct Sound {
    result: std::result::Result<kira::sound::static_sound::StaticSoundData, String>,
}

// impl FontFace {
//     pub fn err(&self) -> Option<String> {
//         self.err.lock().unwrap().as_ref().map(|x| x.clone())
//     }

//     pub fn failed(&self) -> bool {
//         self.err.lock().unwrap().is_some()
//     }

//     pub fn with_internal<R, F>(&self, f: F) -> R
//     where
//         F: FnOnce(&Option<macroquad::text::Font>) -> R,
//     {
//         if self.ready_internal.borrow().is_none() && !self.failed() {
//             let font = self.internal.lock().unwrap();
//             *self.ready_internal.borrow_mut() = font.clone();
//         }
//         f(&self.ready_internal.borrow())
//     }
// }

impl mlua::UserData for Sound {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("error", |_, this| {
            Ok(match &this.result {
                Ok(_) => None,
                Err(err) => Some(err.clone()),
            })
        });
        fields.add_field_method_get("failed", |_, this| Ok(this.result.is_err()));
        fields.add_field_method_get("ready", |_, this| Ok(this.result.is_ok()));
    }
}

#[derive(Clone, Copy)]
struct Surf;

impl mlua::UserData for Surf {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("clear", |_lua, _this, rgb: Option<u32>| {
            // TODO Get target from `this`.
            clear_background(Color::from_hex(rgb.unwrap_or(0)));
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
            |_lua, _this, (text, x, y, font, rgb): (mlua::String, f32, f32, mlua::Value, Option<u32>)| {
                let text = text.to_str()?;
                let mut text_params = TextParams {
                    color: Color::from_hex(rgb.unwrap_or(0xffffff)),
                    font_size: DEFAULT_FONT_SIZE,
                    ..Default::default()
                };
                if let Some(font) = font.as_userdata() {
                    let font = font.borrow::<Font>()?;
                    text_params.font_size = font.size;
                    font.face_internal.with_internal(|face| {
                        let text_params = TextParams {
                            font: face.as_ref(),
                            ..text_params
                        };
                        draw_text_ex(&text, x, y, text_params);
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
    let mut base = PathBuf::from(base_path);
    if base.is_relative() {
        let cwd = env::current_dir().map_err(|e| e.to_string())?;
        base = cwd.join(base);
    }
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

const DEFAULT_FONT_SIZE: u16 = 40;
