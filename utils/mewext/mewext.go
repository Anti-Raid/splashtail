// Extensions to the mewld clustering system
package mewext

import (
	"strconv"

	"github.com/anti-raid/splashtail/utils/syncmap"
	"github.com/cheesycod/mewld/proc"
)

type ClusterExt struct {
	// As the cluster IDs are guaranteed to be constant given a shard ID,
	// we can cache the cluster ID for each shard ID.
	ccache syncmap.Map[uint64, int]
}

func NewClusterExt() *ClusterExt {
	return &ClusterExt{
		ccache: syncmap.Map[uint64, int]{},
	}
}

// Given a shard number, return its cluster ID
func (ce *ClusterExt) GetClusterOfShard(shard uint64, clusterMap []proc.ClusterMap) int {
	if v, ok := ce.ccache.Load(shard); ok {
		return v
	}

	for _, c := range clusterMap {
		for _, s := range c.Shards {
			if s == shard {
				ce.ccache.Store(shard, c.ID)
				return c.ID
			}
		}
	}
	return -1
}

// Given a guild ID, return its shard ID
func (ce *ClusterExt) GetShardIDFromGuildID(guildID string, shardCount int) (uint64, error) {
	gidNum, err := strconv.ParseInt(guildID, 10, 64)

	if err != nil {
		return 0, err
	}

	return uint64(gidNum>>22) % uint64(shardCount), nil
}
