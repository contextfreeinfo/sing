local mod = {}

---@class hi.State
---@field font sys.Font
---@field messages string[]
---@field drawX number
---@field drawY number
---@field lastSoundTime number
---@field sound sys.Sound
---@field soundGap number
---@field stepY number
---@field time number

---@param hub sys.Hub
---@return hi.State
function mod.init(hub)
    local soundGap = 5
    return {
        -- Resources.
        font = hub:fontFace("./assets/Chewy-Regular.ttf"):font(150),
        sound = hub:sound("./assets/ukelele-dolow.ogg"),
        -- Placeholders.
        messages = {},
        drawX = 0,
        drawY = 0,
        lastSoundTime = -soundGap / 2,
        soundGap = soundGap,
        stepY = 0,
        time = 0,
    }
end

---@param hub sys.Hub
---@param state hi.State
function mod.update(hub, state)
    -- Plan text and get sizes.
    local message = "Hi there!"
    local sizeX, sizeY, baselineY = state.font:measure(message)
    local more =
        string.format("Text size: %d x (%d, %d)", sizeX, sizeY, baselineY)
    local moreX = state.font:measure(more)
    -- Figure out what and where to draw.
    state.messages = { message, more }
    state.drawX = (hub.screenSizeX - math.max(sizeX, moreX)) / 2
    state.stepY = sizeY * 1.5
    state.drawY = (hub.screenSizeY - (sizeY + state.stepY)) / 2 + baselineY
    state.time = state.time + hub.frameTime
    -- if state.font.face.ready and state.sound.ready then
    --     -- TODO Pan back and forth playing on an interval.
    --     hub.audio:play(state.sound)
    -- end
    if state.sound.ready then
        if state.time - state.lastSoundTime > state.soundGap then
            hub.audio:play(state.sound)
            state.lastSoundTime = state.time
        end
    end
end

---@param surf sys.Surf
---@param state hi.State
function mod.draw(surf, state)
    if state.font.face.ready then
        -- Got a font, so plan drawing.
        local x, y = state.drawX, state.drawY
        local moveScale = state.stepY * 0.4
        local timeScale = 2
        -- Move in a circle.
        x = x + math.cos(state.time * timeScale) * moveScale
        y = y + math.sin(state.time * timeScale) * moveScale
        -- Draw the messages, one per line.
        for _, message in ipairs(state.messages) do
            surf:text(message, x, y, state.font, 0xffaa00)
            y = y + state.stepY
        end
    end
end

return mod
