local mod = {}

---@param hub sys.Hub
function mod.init(hub)
    local font = hub:font("./font/Chewy-Regular.ttf")
    print(font)
end

return mod
