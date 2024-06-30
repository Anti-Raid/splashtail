// Extensions to the mewld clustering system
package mewext

import (
	"strconv"

	"github.com/cheesycod/mewld/proc"
)

// Given a shard number, return its cluster ID
func GetClusterOfShard(shard uint64, clusterMap []proc.ClusterMap) int {
	for _, c := range clusterMap {
		for _, s := range c.Shards {
			if s == shard {
				return c.ID
			}
		}
	}
	return -1
}

// Given a guild ID, return its shard ID
func GetShardIDFromGuildID(guildID string, shardCount int) (uint64, error) {
	gidNum, err := strconv.ParseInt(guildID, 10, 64)

	if err != nil {
		return 0, err
	}

	return uint64(gidNum>>22) % uint64(shardCount), nil
}

// Given guild ID, return cluster ID
func GetClusterIDFromGuildID(guildID string, clusterMap []proc.ClusterMap, shardCount int) (int, error) {
	shardID, err := GetShardIDFromGuildID(guildID, shardCount)

	if err != nil {
		return 0, err
	}

	return GetClusterOfShard(shardID, clusterMap), nil
}
