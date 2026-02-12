use mlua::{Lua, Result};

fn main() -> Result<()> {
    // 1. Create a new Lua instance
    let lua = Lua::new();

    // 2. Define a simple Lua script
    let script = r#"
        local name = "World"
        print("Hello from Lua 5.5, " .. name .. "!")
    "#;

    // 3. Execute the script
    lua.load(script).exec()?;

    Ok(())
}
