local isInDevMode = false
-- List of users who can use dev commands
local devs = {
    ["728871946456137770"] = true,
    ["564164277251080208"] = true,
    ["787241442770419722"] = true,
}
local cmdLabel = "-dev"

return {
    isInDevMode = isInDevMode,
    devs = devs,
    cmdLabel = cmdLabel
}
