import { RedisClientType } from '@redis/client';
import { createClient } from 'redis';
import { AntiRaid } from './client';

/*
type LauncherCmd struct {
	Scope     string         `json:"scope"`
	Action    string         `json:"action"`
	Args      map[string]any `json:"args,omitempty"`
	CommandId string         `json:"command_id,omitempty"`
	Output    any            `json:"output,omitempty"`
	Data      map[string]any `json:"data,omitempty"` // Used in action logs
}
*/
export interface LauncherCmd {
    scope: string
    action: string
    args?: { [key: string]: any }
    command_id?: string
    output?: any
    data?: { [key: string]: any }
}

export class BotRedis {
    client: RedisClientType | null = null
    mewld_pubsub: RedisClientType | null = null
    bot: AntiRaid

    constructor (bot: AntiRaid) {
        this.bot = bot
    }

    async load() {
        await this.startRedis()
        await this.startMewld()
    }

    async launchNext() {
        let launchNext: LauncherCmd = {
            scope: "launcher",
            action: "launch_next",
            args: {
                id: this.bot.clusterId,
            }
        }

        await this.client.publish(process.env.MEWLD_CHANNEL, JSON.stringify(launchNext))
    }

    async startRedis() {
        this.client = createClient()
        await this.client.connect()
    }

    async startMewld() {
        this.mewld_pubsub = this.client.duplicate()

        this.mewld_pubsub.on("error", (err) => {
            this.bot.logger.error("Redis", err)
        })
    
        await this.mewld_pubsub.connect()

        await this.mewld_pubsub.subscribe(process.env.MEWLD_CHANNEL, async (message: string, channel: string) => {
            this.bot.logger.info("Redis", "Received message", message, channel)

            try {
                let data = JSON.parse(message)

                // Diagnostics payload
                if(data?.diag && data?.id == this.bot.clusterId) {
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
                    let shardHealthData = []

                    this.bot.logger.info("Redis", "Have current shards", this.bot.ws.shards)

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
                        this.bot.logger.error("Redis", "Unimplemented payload", data)
                        return
                    }
                    
                    let payload: LauncherCmd = data

                    if(payload.scope == "bot") {
                        switch (payload.action) {
                            case "all_clusters_launched":
                                this.bot.allClustersLaunched = true
                        }
                    }
                }
            } catch (err) {
                this.bot.logger.error("Redis", "Error responding to core mewld payload", err)
            }
        })
    }
}