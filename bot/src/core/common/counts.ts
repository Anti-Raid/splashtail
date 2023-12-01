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
    let guildCacheSize = await client.shard.fetchClientValues('guilds.cache.size')

    let totalGuilds = guildCacheSize.reduce((acc: number, guildCount: number) => acc + guildCount, 0);

    return totalGuilds
}

export async function getUserCount(client: AntiRaid) {
    let memberCount = await client.shard.broadcastEval(c => c.guilds.cache.reduce((acc, guild) => acc + guild.memberCount, 0))

    let totalMembers = memberCount.reduce((acc: number, memberCount: number) => acc + memberCount, 0);

    return totalMembers
}

export async function getShardCount(client: AntiRaid) {
    return client.shard.count
    return client
}