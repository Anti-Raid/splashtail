# Example Templates

To help you get started with templating, we have provided a few examples below along with explanations of what/how they work

## Example 1: Simple Audit Logging

### Explanation

#### 1. Pragma

The first line of the template is a pragma. This is a special statement beginning with ``@pragma`` or ``-- @pragma`` that tells AntiRaid what language the template is written in, what options to use, and how tools such as CI, websites and other automation should handle your template. The rest of the pragma is a JSON object that contains the options. Note that if you do not provide a pragma, a default one will be used, however this will not allow you to provide capabilities to your template such as sending a message on Discord etc.

In this case, we want to tell AntiRaid that we are coding a template in Lua and that we want to allow the capability to send messages to Discord. This is done by adding the ``allowed_caps`` key to the pragma and specifying the capabilities we want to allow as seen below:

```lua
-- @pragma {"lang":"lua","allowed_caps":["discord:create_message"]}
```

#### 2. Creating a message

Next, we need to extract the arguments and token from the context. The arguments are passed to the template when it is executed and contain all the data we need to work with. The token is used to authenticate the template and gain access to the templates context in privileged AntiRaid API's. All of this is provided using variable arguments.

```lua
local args, token = ...
```

There are 3 things we want to import for this template to work. The first is the Discord module, which allows us to send messages to Discord. The second is the Interop module, which provides some functions allowing for seamless interoperability between your template and AntiRaid. The third is the promise module which lets us run asynchronous tasks like sending a message to Discord.

```lua
local discord = require "@antiraid/discord"
local interop = require "@antiraid/interop"
local promise = require "@antiraid/promise"
```


Next, we create the embed using the events ``title`` field as the embed title and an empty description.

```lua
-- Make the embed
local embed = {
    title = args.title, 
    description = "", -- Start with empty description
    fields = {}, -- Start with empty fields
}
```

**NOTE: You can use the [API Reference](./2-plugins.md) to see what functions are available in the AntiRaid SDK**

A quick side track here. When coding in Lua, tables can be iterated over using the builtin ``pairs`` function like below:

```lua
for key, value in pairs(my_table) do
    -- Do something with key and value
end
```

#### 3. Creating the audit log message fields etc.

The next step is to add fields to the embed by actually handling the Discord events we want properly. This is pretty annoying to do without some typing and helper methods though...

Fortunately, AntiRaid has a solution: [templating-types](https://github.com/Anti-Raid/templating-types)! This does require extra effort in the form of bundling with [templating-template](https://github.com/Anti-Raid/templating-template) as a template. See [our guide](./4-bundling.md) for more information on how to do this.

Once you're done making the message as you'd like. It's time to send the message!

#### 4. Sending the message

Finally, we can send the message using the Discord module. To do this, we need to create a message object and set the embeds property to an array containing our embed. This is also where interop comes in handy, as we need to set the metatable of the embeds array to the interop array metatable so AntiRaid knows that embeds is an array. Next, we use the Discord plugin to make a new Discord action executor which then lets us send the message to the specified channel.

```lua
setmetatable(message.embeds, interop.array_metatable)

table.insert(message.embeds, embed)

-- Send message using action executor
local discord_executor = discord.new(token);
discord_executor:create_message({
    channel_id = "CHANNEL_ID_HERE",
    message = message
})
```

### 5. (Optional) Key-value

With ``@antiraid/kv``, you can save the channel id to a key-value store in the website and then fetch it like so:

```lua
local kv = require "@antiraid/kv"

local kvExecutor = kv.new(token)
local channelId = kvExecutor:get("auditlog_channel")
```