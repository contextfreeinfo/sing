local mod = {}

-- TODO Unit tests that these things fail or work as expected.
-- local file = io.open("hi.txt", "w")
-- local a = coroutine.running()

---@class Sprite
---@field dirX number
---@field dirY number
---@field posX number
---@field posY number
---@field sizeX number
---@field sizeY number
---@field speed number

---@class State
---@field sprites Sprite[]

---@param hub sys.Hub
---@return State
function mod.init(hub)
    local sprites = {}
    for _ = 1, 1000 do
        table.insert(sprites, {
            dirX  = math.random(-1, 1),
            dirY  = math.random(-1, 1),
            posX  = math.random(0, 1920), -- hub.screenSizeX),
            posY  = math.random(0, 1080), -- hub.screenSizeY),
            sizeX = 40,
            sizeY = 40,
            speed = math.random(100, 500),
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
        local move = hub.frameTime * sprite.speed
        sprite.posX = sprite.posX + move * sprite.dirX
        sprite.posY = sprite.posY + move * sprite.dirY
        -- Check collision
        local x1 = sprite.posX + sprite.sizeX
        local y1 = sprite.posY + sprite.sizeY
        if sprite.posX < 0 then
            sprite.dirX = 1
        elseif x1 > hub.screenSizeX then
            sprite.dirX = -1
        end
        if sprite.posY < 0 then
            sprite.dirY = 1
        elseif y1 > hub.screenSizeY then
            sprite.dirY = -1
        end
    end
end

---@param surf sys.Surf
---@param state State
function mod.draw(surf, state)
    surf:clear(0x104060)
    for _, sprite in ipairs(state.sprites) do
        local x1 = sprite.posX + sprite.sizeX
        local y1 = sprite.posY + sprite.sizeY
        surf:rect(sprite.posX, sprite.posY, x1, y1, 0x205080)
    end
end

return mod
