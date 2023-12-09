import { RedisClientType } from '@redis/client';
import { createClient } from 'redis';
import { AntiRaid } from './client';
import { Status } from 'discord.js';
import EventEmitter from 'events';
import { randomBytes } from 'crypto';
import { KV } from './coreTypes/kv';
import { Task, TaskCreateResponse } from '../generatedTypes/types';

export interface DiagResponse {
    Nonce: string
    Data: ShardHealth[]
}

export interface ShardHealth {
    shard_id: number
    up: boolean
    latency: number
    guilds: number
    users: number
}

/*
type LauncherCmd struct {
	Scope     string         `json:"scope"`
	Action    string         `json:"action"`
	Args      map[string]any `json:"args,omitempty"`
	CommandId string         `json:"command_id,omitempty"`
	Output    any            `json:"output,omitempty"`
	Data      map[string]any `json:"data,omitempty"` // Used in action logs
}

For a response to a bot-scoped IPC command, set data?.respCluster to the cluster ID that sent the command

If a request/response is cluster-specific, set data?.targetCluster to the cluster IDs that should receive the response

For a splashtail scoped IPC command, respCluster must be -1 and originCluster (in request) should be set to the cluster ID that sent the command
*/
export interface LauncherCmd {
    scope: string
    action: string
    args?: KV
    command_id?: string
    output?: any
    data?: KV
}

/**
 * Options for IPC send/respond
 */
export interface IpcSendOptions {
    /**
     * The cluster ID that should recieve the response
     */
    targetCluster?: number[]
}

/**
 * Options for IPC fetch
 */
export interface IpcFetchOptions {
    /**
     * Timeout in milliseconds. If not set, will wait for 10000ms (10 seconds)
     */
    timeout?: number
    /**
     * The number of clusters that need to respond before the promise resolves. If not set, will wait for all clusters to respond or if scope is splashtail, will set to 1
     */
    numClustersNeeded?: number
}

/**
 * IPCCommand is the definition of a command that can be sent over IPC to yield a response
 */
export interface IPCCommand {
    action: string
    command: (ctx: IPCContext) => Promise<void>
}

/**
 * TaskPollOptions is the options for polling for a task
 */
export interface TaskPollOptions {
    /**
     * The number of milliseconds to wait for a response before timing out
     */
    timeout: number

    /**
     * The target ID who is requesting the task. Needed for Access Control
     */
    targetId: string

    /**
     * The target type who is requesting the task. Needed for Access Control
     */
    targetType: "User" | "Server"
    
    /**
     * The callback
     */
    callback: (task: Task) => Promise<void>
}

export class BotRedis extends EventEmitter {
    client: RedisClientType | null = null
    bot: AntiRaid
    ipcs: Map<string, IPCCommand>;
    private mewld_notifier: RedisClientType | null = null
    private ipcCommandQueue: Map<string, IPCRequestHandle>;

    constructor (bot: AntiRaid) {
        super()

        this.bot = bot
        this.ipcs = new Map()
        this.ipcCommandQueue = new Map()
    }

    async handleIpcQueue(cmd: LauncherCmd) {
        for(let [id, handle] of this.ipcCommandQueue) {
            if(!handle) {
                this.bot.logger.info("Redis [IPCQueueSweep]", "Null handle", id)
                this.ipcCommandQueue.delete(id)
                continue
            }

            if(!handle.isPending()) {
                this.bot.logger.info("Redis [IPCQueueSweep]", "Not pending handle", id)
                handle.stop()
                this.ipcCommandQueue.delete(id)
                continue
            }

            handle.onResp(cmd)

            // If the handle is now no longer pending, stop it and remove it from the queue
            if(!handle.isPending()) {
                handle.stop()
                this.ipcCommandQueue.delete(id)
                continue
            }
        }
    }

    async load() {
        await this.startRedis()
        await this.startMewld()
    }

    /**
     * Creates an action log on mewld
     */
    async createMewldActionLog(event: string, data: KV) {
        let payload: LauncherCmd = {
            scope: "launcher",
            action: "action_logs",
            data: data || {}
        }

        payload.data["event"] = event

        await this.client.publish(process.env.MEWLD_CHANNEL, JSON.stringify(payload))
    }
    
    /**
     * Sends an IPC request to all clusters
     * @param payload The payload to send
     */
    async sendIpcRequest(payload: LauncherCmd, opts?: IpcSendOptions, fetchOpts?: IpcFetchOptions): Promise<IPCRequestHandle | null> {
        // Ensure payload?.data?.respCluster is unset
        if(payload?.data?.respCluster) {
            delete payload.data.respCluster
        }

        payload.data = payload.data || {}

        // Set targetCluster if scope is bot
        if(opts?.targetCluster && payload.scope == "bot") {
            payload.data.targetCluster = opts.targetCluster
        }

        // Set command ID if not set
        if(!payload.command_id) {
            payload.command_id = randomBytes(20).toString('hex');
        }

        let handle: IPCRequestHandle | null = null

        if(fetchOpts) {
            fetchOpts.numClustersNeeded = fetchOpts?.numClustersNeeded || ((payload?.scope == "splashtail") ? 1 : this.bot.clusterCount)
            fetchOpts.timeout = fetchOpts?.timeout || 10000

            // Create handle
            handle = new IPCRequestHandle(this.bot, this, payload, fetchOpts)
            this.ipcCommandQueue.set(payload.command_id, handle)
        }

        // Set channel for IPC if scope is splashtail
        let channel = process.env.MEWLD_CHANNEL
        if(payload.scope == "splashtail") {
            channel = `${process.env.MEWLD_CHANNEL}/${this.bot.clusterId}`
        }

        // Publish
        try {
            await this.client.publish(channel, JSON.stringify(payload))
        } catch (err) {
            if(handle) {
                handle.stop()
            }
            throw err
        }

        return handle
    }

    /**
     * 
     * @param tcr The task create response
     * @param opts The options for polling for a task
     */
    async pollForTask(tcr: TaskCreateResponse, opts: TaskPollOptions) {
        let done = false
        let handle: IPCRequestHandle | null = null
        let task: Task | null = null
        let start_from = 0
        let taskStatuses: KV[] = []

        let tcrB = JSON.stringify(tcr) // Optimization to avoid constant serialization

        while(task?.state != "completed" && !done) {
            handle = await this.sendIpcRequest({
                scope: "splashtail",
                action: "get_task",
                data: {
                    target_id: opts.targetId,
                    target_type: opts.targetType,
                    task: tcrB,
                    start_from
                }
            }, null, {
                timeout: opts.timeout,
            })

            if(!handle) throw new Error("Invalid IPC handle")

            let rmap = await handle.fetch()

            if(rmap.size == 0 || !rmap.has(-1)) {
                throw new Error("No response from co-ordinator server")
            }

            let resp = rmap.get(-1)

            if(resp?.output?.error) {
                throw new Error(resp?.output?.error)
            }

            task = resp?.output

            if(!task || !task?.task_id) {
                throw new Error("No task returned")
            }
            
            // Set new statuses
            taskStatuses = [
                ...taskStatuses,
                ...(task?.statuses || [])
            ]

            task.statuses = taskStatuses

            await opts.callback(task)

            if(task?.state == "completed" || task?.state == "failed") {
                return task
            }

            start_from = (task?.statuses?.length) || 0
        }

        return task // Return the task till what we have or whatever we have left of a task
    }

    /**
     * Signals to mewld to launch the next cluster
     */
    async mewldLaunchNext() {
        let launchNext: LauncherCmd = {
            scope: "launcher",
            action: "launch_next",
            args: {
                id: this.bot.clusterId,
            }
        }

        await this.client.publish(process.env.MEWLD_CHANNEL, JSON.stringify(launchNext))
    }

    /**
     * Starts the base redis client
     */
    private async startRedis() {
        this.client = createClient()
        await this.client.connect()
    }

    private async ipcHandler(payload: LauncherCmd) {
        // Only bot-scoped IPC handling is supported. The rest should be handled directly in startMewld listener
        if(payload.scope == "bot") {
            let ipcCommand = this.ipcs.get(payload.action)

            if(!ipcCommand) {
                this.bot.logger.warn("Redis [IPC]", "Unimplemented IPC command", payload)
                return
            }

            let ipcCtx = new IPCContext(this.bot, this, payload)

            await ipcCommand.command(ipcCtx)
        }
    }

    /**
     * The actual IPC implementation
     */
    private async ipcEmitter(message: string, channel: string) {
        try {
            let data = JSON.parse(message)

            // Diagnostics payload
            if(data?.diag) {
                /*type DiagResponse struct {
                    Nonce string        // Random nonce used to validate that a nonce comes from a specific diag request
                    Data  []ShardHealth // The shard health data

                    type ShardHealth struct {
                        ShardID uint64  `json:"shard_id"` // The shard ID
                        Up      bool    `json:"up"`       // Whether or not the shard is up
                        Latency float64 `json:"latency"`  // Latency of the shard (optional, send if possible)
                        Guilds  uint64  `json:"guilds"`   // The number of guilds in the shard
                    }
                }*/
                if(data?.id != this.bot.clusterId) {
                    return
                }

                // Collect shard health
                let shardHealthData: ShardHealth[] = []

                for(let [id, shard] of this.bot.ws.shards) {
                    shardHealthData.push({
                        shard_id: id,
                        up: shard.status == Status.Ready,
                        latency: shard.ping,
                        guilds: this.bot.guilds.cache.filter(g => g.shardId == id).size,
                        users: this.bot.guilds.cache.filter(g => g.shardId == id).reduce((acc, g) => acc + g.memberCount, 0)
                    })
                }

                // This gets quite spammy...
                if(process.env.DEBUG_SHARDS) {
                    this.bot.logger.debug("Redis", "Have current shards", this.bot.ws.shards)
                }

                let resp: LauncherCmd = {
                    scope: "launcher",
                    action: "diag",
                    output: JSON.stringify({
                        Nonce: data.nonce,
                        Data: shardHealthData
                    })
                }    
                
                await this.client.publish(process.env.MEWLD_CHANNEL, JSON.stringify(resp))
            } else {
                if(!data?.action) {
                    if(data?.output == "ok" && data?.scope == "bot") {
                        this.emit("ipcAcknowledge", data)
                        return
                    }

                    this.bot.logger.error("Redis", "Unimplemented payload [not oneof diag or action payload)", data)
                    return
                }
                
                let payload: LauncherCmd = data

                if(Array.isArray(payload?.data?.targetCluster) && !payload?.data?.targetCluster.includes(this.bot.clusterId)) {
                    // This response is not for us, ignore it
                    return
                }

                if(payload?.data?.respCluster != undefined && payload?.data?.respCluster != null) {
                    if(this.ipcCommandQueue.has(payload?.command_id)) {
                        this.handleIpcQueue(payload)
                    }

                    return
                }

                if(process.env.IPC_DEBUG == "true") {
                    this.bot.logger.info("Redis [IPC]", "Received launcherCmd payload", channel, payload)
                }

                if(payload.scope == "bot") {
                    try {
                        await this.ipcHandler(payload)
                    } catch (err) {
                        this.bot.logger.error("Redis [IPC]", "Error handling ipc payload", err)
                    }
                } else if(payload.scope == "launcher") {
                    switch (payload.action) {
                        case "diag":
                            // We have recieved a diagnostic payload from other clusters, save it
                            let diagResp: DiagResponse = JSON.parse(payload.output)
                            if(Array.isArray(diagResp.Data)) {
                                for(let shard of diagResp.Data) {
                                    this.bot.currentShardHealth.set(shard.shard_id, shard)
                                }
                            }
                        default:
                            if(process.env.IPC_DEBUG == "true") {
                                this.bot.logger.info("Redis [IPC]", "Received launcherCmd payload", channel, payload)
                            }
                    }
                }
            }
        } catch (err) {
            this.bot.logger.error("Redis", "Error responding to core mewld payload", err)
        }
    }

    /**
     * Starts the mewld notification client
     */
    private async startMewld() {
        this.mewld_notifier = this.client.duplicate()

        this.mewld_notifier.on("error", (err) => {
            this.bot.logger.error("Redis", err)
        })
    
        await this.mewld_notifier.connect()

        // There are two channels that we need to subscribe to
        // 
        // The first is the common channel, which is used for normal sends
        //
        // The other is the cluster-specific channel, which is used for IPC with splashtail (these payloads are much larger)
        await this.mewld_notifier.subscribe([process.env.MEWLD_CHANNEL, `${process.env.MEWLD_CHANNEL}/${this.bot.clusterId}`], (message: string, channel: string) => this.ipcEmitter(message, channel))
    }
}

/**
 * A handle to a request created by a IPC
 */
export class IPCRequestHandle {
    bot: AntiRaid
    redis: BotRedis
    private request: LauncherCmd
    private commandId: string
    private done: boolean;
    ipcQueue: Map<number, LauncherCmd>
    fetchOpts: IpcFetchOptions

    constructor(bot: AntiRaid, redis: BotRedis, request: LauncherCmd, fetchOpts: IpcFetchOptions) {
        if(!bot) {
            throw new Error("Invalid bot")
        }
        if(!redis) {
            throw new Error("Invalid redis")
        }
        if(!request || !request.command_id) {
            throw new Error("Invalid request")
        }
        if(!fetchOpts) {
            throw new Error("Invalid fetchOpts")
        }

        this.bot = bot
        this.redis = redis
        this.request = request
        this.commandId = request.command_id
        this.ipcQueue = new Map()
        this.fetchOpts = fetchOpts
    }

    stop() {
        this.done = true
    }

    onResp(resp: LauncherCmd) {
        if(process.env.IPC_DEBUG == "true") {
            this.bot.logger.debug("IPCRequestHandle", "Got response", resp, this.commandId)
        }
        
        if(resp.command_id == this.commandId && resp.scope == this.request.scope && resp.action == this.request.action) {
            this.ipcQueue.set(resp.data.respCluster, resp)
        }
    }

    /**
     * Whether or not the request is pending
     * @returns Whether or not the request is pending
     */
    isPending() {
        if(this.done) return false
        if(this.fetchOpts.numClustersNeeded == -1 && this.ipcQueue.size != 0) return false
        return this.ipcQueue.size < this.fetchOpts.numClustersNeeded
    }

    /**
     * Fetches the response to the request
     */
    async fetch(): Promise<Map<number, LauncherCmd>> {
        // Wait for all clusters to respond
        while(!this.done && this.isPending()) {
            await new Promise((resolve) => setTimeout(resolve, 100))
        }

        return this.ipcQueue
    }
}

/**
 * A context for an IPC command that should be handled
 */
export class IPCContext {
    bot: AntiRaid
    redis: BotRedis
    request: LauncherCmd

    constructor(bot: AntiRaid, redis: BotRedis, request: LauncherCmd) {
        this.bot = bot
        this.redis = redis
        this.request = request
    }

    async respond(payload: LauncherCmd, opts?: IpcSendOptions) {
        if(payload.action != this.request.action || payload.scope != this.request.scope) {
            throw new Error("Cannot respond with a different action/scope")
        }
        

        payload.data = payload?.data || {}

        // Set respCluster
        payload.data.respCluster = this.bot.clusterId

        // Set command ID if in request
        if(this.request.command_id) {
            payload.command_id = this.request.command_id
        }

        // Set targetCluster
        if(opts?.targetCluster) {
            payload.data.targetCluster = opts.targetCluster
        }

        // If targetCluster is set in request data, set it in response data
        if(this.request.data?.targetCluster) {
            payload.data.targetCluster = this.request.data.targetCluster
        }

        // Publish
        await this.redis.client.publish(process.env.MEWLD_CHANNEL, JSON.stringify(payload))
    }
}