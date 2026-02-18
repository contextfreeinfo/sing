---@meta

--- Global system namespace
---@class sys
sys = {}


---===
--- Font handle
---@class sys.Font
sys.Font = {}
---===


---===
--- Access to system input, output, and state
---@class sys.Hub
sys.Hub = {}

--- Load a font from a resource reference
---@param resource string | sys.Resource
---@return sys.Font
function sys.Hub:font(resource) end

--- The delta time for the current frame
sys.Hub.frame_time = 0

--- Screen width in pixels
sys.Hub.screen_size_x = 0

--- Screen height in pixels
sys.Hub.screen_size_y = 0
---===


---===
--- Abstract resource handle
--- TODO This could allow for prep. Good idea?
---@class sys.Resource
sys.Resource = {}
---===


---@alias sys.Rgb integer # A 24-bit RGB color (e.g., 0xFF0000)


---===
--- Surface used for drawing graphics
---@class sys.Surf
sys.Surf = {}

--- Clear the screen with a specific color
---@param rgb sys.Rgb
function sys.Surf:clear(rgb) end

--- Fill a rectangle
---@param x0 number
---@param y0 number
---@param x1 number
---@param y1 number
---@param rgb sys.Rgb
function sys.Surf:rect(x0, y0, x1, y1, rgb) end
---===
