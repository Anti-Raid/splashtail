// AI converted using https://www.codeconvert.ai/free-converter

import { ClusterMap } from "$lib/generated/mewld/proc";

export function getClusterOfShard(shard: number, clusterMap: ClusterMap[]): number {
    for (const c of clusterMap) {
        for (const s of c.Shards) {
            if (s === shard) {
                return c.ID;
            }
        }
    }
    return -1;
}

export function getShardIDFromGuildID(guildID: string, shardCount: number): [number, Error | null] {
    let gidNum: bigint;
    try {
        gidNum = BigInt(guildID);
    } catch (err) {
        return [0, err as Error];
    }
    return [Number(gidNum >> BigInt(22)) % shardCount, null];
}