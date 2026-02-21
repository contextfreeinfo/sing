---@meta


--- Global system namespace.
---@class sys
sys = {}
---===


---===
--- Sized font, where each costs vram. Limit how many you make as well as the
--- size of each.
---@class sys.Font
--- The face this font comes from.
---@field face sys.FontFace
--- The face this font comes from.
---@field size integer
sys.Font = {}

--- Measure sizes for the given text.
--- TODO Throw if not ready?
---@param text string
---@return number size_x
---@return number size_y
---@return number baseline_y
function sys.Font:measure(text) end
---===


---===
--- Font face, which is independent of size.
---@class sys.FontFace
--- Whether the font face is properly loaded.
---@field ready boolean
--- The error message if any.
---@field error? string
sys.FontFace = {}

--- Prepare a font at a particular size. Each costs vram. Bigger costs more.
--- Limit how many you make as well as the size. If the face isn't ready, the
--- font also won't be.
--- TODO Preconfigure which sizes to use for which sizes on screen?
--- TODO Different atlas per size would be nice.
---@param size number
---@return sys.Font
function sys.FontFace:font(size) end
---===


---===
--- Access to system input, output, and state.
---@class sys.Hub
sys.Hub = {}

--- Load a font from a relative path.
--- TODO Some way to manage multiface for different code ranges?
--- TODO Size ratios for different faces in a multiface?
---@param path string
---@return sys.FontFace
function sys.Hub:font_face(path) end

--- Define a font face and scale.

--- The delta time for the current frame.
sys.Hub.frame_time = 0

--- Screen width in pixels.
sys.Hub.screen_size_x = 0

--- Screen height in pixels.
sys.Hub.screen_size_y = 0
---===


---@alias sys.Rgb integer # A 24-bit RGB color (e.g., 0xFF0000)


---===
--- Surface used for drawing graphics.
---@class sys.Surf
sys.Surf = {}

--- Clear the screen with a specific color.
---@param rgb? sys.Rgb Defaults to black.
function sys.Surf:clear(rgb) end

--- Fill a rectangle.
---@param x0 number
---@param y0 number
---@param x1 number
---@param y1 number
---@param rgb sys.Rgb
function sys.Surf:rect(x0, y0, x1, y1, rgb) end

--- Draw text to the screen.
---@param text string
---@param x number
---@param y number
---@param font? sys.Font Defaults to internal font at size 40.
---@param rgb? sys.Rgb Defaults to white.
--- TODO Some size that auto applies scale?
function sys.Surf:text(text, x, y, font, rgb) end

---===
