local interop = require "@antiraid/interop"

local captcha_config = {}

-- Basic options
captcha_config.char_count = 7
captcha_config.filters = {}
setmetatable(captcha_config.filters, interop.array_metatable) -- Filters is an array
captcha_config.viewbox_size = { 280, 160 }
setmetatable(captcha_config.viewbox_size, interop.array_metatable) -- Viewbox size is a tuple

-- Add noise filter
local noise_filter = {
    filter = "Noise",
    prob = 0.05
}

table.insert(captcha_config.filters, noise_filter)

-- Add wave filter
local wave_filter = {
    filter = "Wave",
    f = 4.0, -- Frequency
    amp = 20.0, -- Amplitude
    d = "horizontal" -- Direction
}

table.insert(captcha_config.filters, wave_filter)

-- Add grid filter
local grid_filter = {
    filter = "Grid",
    x_gap = 10,
    y_gap = 30
}

table.insert(captcha_config.filters, grid_filter)

-- Add line filter
local line_filter = {
    filter = "Line",
    p1 = setmetatable({ 0.0, 0.0 }, interop.array_metatable),
    p2 = setmetatable({ 30.0, 100.0 }, interop.array_metatable),
    thickness = 7.0,
    color = setmetatable({ 0, 0, 0 }, interop.array_metatable)
}

table.insert(captcha_config.filters, line_filter)

-- Add color invert filter
local color_invert_filter = {
    filter = "ColorInvert"
}

table.insert(captcha_config.filters, color_invert_filter)

-- Add random line filter
local random_line_filter = {
    filter = "RandomLine"
}

table.insert(captcha_config.filters, random_line_filter)

return captcha_config
