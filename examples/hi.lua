local mod = {}

---@class State
---@field font sys.Font

---@param hub sys.Hub
---@return State
function mod.init(hub)
    return {
        font = hub:font("./font/Chewy-Regular.ttf"),
    }
end

---@param surf sys.Surf
---@param state State
function mod.draw(surf, state)
    if state.font.ready then
        -- TODO Need a font scale option?
        surf:text("Hi there!", 300, 400, state.font, 200, 0xffaa00)
    end
end

return mod
