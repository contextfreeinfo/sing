local mod = {}

-- local file = io.open("hi.txt", "w")
-- local a = coroutine.running()

---@param surf sys.Surf
function mod.draw(surf)
    surf:clear(0x104060)
end

return mod
