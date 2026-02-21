local mod = {}

---@class State
---@field font sys.Font

---@param hub sys.Hub
---@return State
function mod.init(hub)
    return {
        font = hub:font_face("./font/Chewy-Regular.ttf"):font(200),
    }
end

---@param surf sys.Surf
---@param state State
function mod.draw(surf, state)
    if state.font.face.ready then
        surf:text("Hi there!", 300, 400, state.font, 0xffaa00)
        -- surf:text("Hi there!", 300, 400)
    end
end

return mod
