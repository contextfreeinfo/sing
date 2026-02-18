local mod = {}

-- TODO Unit tests that these things fail or work as expected.
-- local file = io.open("hi.txt", "w")
-- local a = coroutine.running()

---@class Sprite
---@field dir_x number
---@field dir_y number
---@field pos_x number
---@field pos_y number
---@field size_x number
---@field size_y number
---@field speed number

---@class State
---@field sprites Sprite[]

---@return State
function mod.init()
    local sprites = {}
    for _ = 1, 1000 do
        table.insert(sprites, {
            dir_x  = math.random(-1, 1),   -- TODO Just -1 or 1
            dir_y  = math.random(-1, 1),
            pos_x  = math.random(0, 1920), -- TODO Use hub for size?
            pos_y  = math.random(0, 1080),
            size_x = 40,
            size_y = 40,
            speed  = math.random(100, 500),
        })
    end
    return {
        sprites = sprites
    }
end

---@param hub sys.Hub
---@param state State
function mod.update(hub, state)
    for _, sprite in ipairs(state.sprites) do
        -- Move
        local move = hub.frame_time * sprite.speed
        sprite.pos_x = sprite.pos_x + move * sprite.dir_x
        sprite.pos_y = sprite.pos_y + move * sprite.dir_y
        -- Check collision
        local x1 = sprite.pos_x + sprite.size_x
        local y1 = sprite.pos_y + sprite.size_y
        if sprite.pos_x < 0 then
            sprite.dir_x = 1
        elseif x1 > hub.screen_size_x then
            sprite.dir_x = -1
        end
        if sprite.pos_y < 0 then
            sprite.dir_y = 1
        elseif y1 > hub.screen_size_y then
            sprite.dir_y = -1
        end
    end
end

---@param surf sys.Surf
---@param state State
function mod.draw(surf, state)
    surf:clear(0x104060)
    for _, sprite in ipairs(state.sprites) do
        local x1 = sprite.pos_x + sprite.size_x
        local y1 = sprite.pos_y + sprite.size_y
        surf:rect(sprite.pos_x, sprite.pos_y, x1, y1, 0x205080)
    end
end

return mod
