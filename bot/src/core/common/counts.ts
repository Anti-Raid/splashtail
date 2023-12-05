/**
 * @file counts.ts
 * @description Functions for getting the total guild count and other counts
 */

import { AntiRaid } from "../client";

/**
 * Returns the total guild count of the client from all shards
 * @param client The client to get the count from
 * @returns The total guild count of the client from all shards
 */
export async function getServerCount(client: AntiRaid) {
    if(!client.currentShardHealth || !client.currentShardHealth.size) {
        throw new Error("Shard health not initialized")
    }

    client.logger.info("GetServerCount", client.currentShardHealth)

    let totalGuilds = 0

    for(let [_, sh] of client.currentShardHealth) {
        totalGuilds += sh.guilds || 0
    }

    return totalGuilds
}

export async function getUserCount(client: AntiRaid) {
    if(!client.currentShardHealth || !client.currentShardHealth.size) {
        throw new Error("Shard health not initialized")
    }

    let totalUsers = 0

    for(let [_, sh] of client.currentShardHealth) {
        totalUsers += sh.users || 0
    }

    return totalUsers
}

export async function getShardCount(client: AntiRaid) {
    if(!client.currentShardHealth || !client.currentShardHealth.size) {
        throw new Error("Shard health not initialized")
    }

    return client.currentShardHealth.size || 0
}