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

    loop {
        clear_background(BLUE);
        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, WHITE);
        draw_text("Hi there!", 20.0, 20.0, 30.0, WHITE);
        next_frame().await
    }
}
