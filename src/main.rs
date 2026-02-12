use macroquad::prelude::*;
use mlua::{Lua, Result};

fn window_conf() -> Conf {
    Conf {
        window_title: "Sing".into(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<()> {
    // 1. Create a new Lua instance
    let lua = Lua::new();

    // 2. Define a simple Lua script
    let script = r#"
        local name = "World"
        print("Hello from Lua 5.5, " .. name .. "!")
    "#;

    // 3. Execute the script
    lua.load(script).exec()?;

    let mut pos = Vec2::ZERO;
    let size = Vec2::new(40.0, 40.0);
    let mut dir = Vec2::new(1.0, 1.0);
    let speed = 400.0;
    let screen_size = Vec2::new(screen_width(), screen_height());
    loop {
        clear_background(BLUE);
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
