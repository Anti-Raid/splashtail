import { IPCCommand, IPCContext } from "../redis";

let ipc: IPCCommand = {
    action: "get_user_count",
    command: async (ctx: IPCContext) => {
        await ctx.respond({
            scope: "bot",
            action: "get_user_count",
            data: {
                count: ctx.bot.guilds.cache.reduce((acc, guild) => acc + guild.memberCount, 0)
            }
        })
    }
}

export default ipc