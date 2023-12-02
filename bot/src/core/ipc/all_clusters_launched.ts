import { IPCCommand, IPCContext } from "../redis";

let ipc: IPCCommand = {
    action: "all_clusters_launched",
    command: async (ctx: IPCContext) => {
        ctx.bot.logger.info("IPC (AllClustersLaunched)", "All clusters have launched")
        ctx.bot.allClustersLaunched = true
    }
}

export default ipc