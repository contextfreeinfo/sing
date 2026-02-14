local mod = {}

-- TODO Unit tests that these things fail or work as expected.
-- local file = io.open("hi.txt", "w")
-- local a = coroutine.running()

---@class State
---@field dir_x number
---@field dir_y number
---@field pos_x number
---@field pos_y number
---@field size_x number
---@field size_y number
---@field speed number

---@return State
function mod.init()
    return {
        dir_x = 1,
        dir_y = 1,
        pos_x = 0,
        pos_y = 0,
        size_x = 40,
        size_y = 40,
        speed = 400,
    }
end

---@param hub sys.Hub
---@param state State
function mod.update(hub, state)
    -- Move
    local move = hub.frame_time * state.speed
    state.pos_x = state.pos_x + move * state.dir_x
    state.pos_y = state.pos_y + move * state.dir_y
    -- Check collision
    local x1 = state.pos_x + state.size_x
    local y1 = state.pos_y + state.size_y
    if state.pos_x < 0 then
        state.dir_x = 1
    elseif x1 > hub.screen_size_x then
        state.dir_x = -1
    end
    if state.pos_y < 0 then
        state.dir_y = 1
    elseif y1 > hub.screen_size_y then
        state.dir_y = -1
    end
end

---@param surf sys.Surf
---@param state State
function mod.draw(surf, state)
    surf:clear(0x104060)
    local x1 = state.pos_x + state.size_x
    local y1 = state.pos_y + state.size_y
    surf:rect(state.pos_x, state.pos_y, x1, y1, 0xffffff)
end

return mod
