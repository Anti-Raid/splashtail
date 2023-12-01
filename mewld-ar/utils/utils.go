package utils

import (
	"bufio"
	"mewld/coreutils"
	"mewld/proc"
	"os"
)

func ReadLines(path string) ([]string, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	var lines []string
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		lines = append(lines, scanner.Text())
	}
	return lines, scanner.Err()
}

// Given a shard count, return the shards for each cluster (128 would be [[0, 1, ..., 9], [10, 11, ..., 19]])
// However, if the shard count is not a multiple of the number of clusters, the last cluster will have fewer shards etc.
// So, 1 would mean [[0]]
func GetClusterList(clusterNames []string, shards uint64, perCluster uint64) []proc.ClusterMap {
	var clusterMap []proc.ClusterMap

	var shardArr []uint64
	var cid int = -1 // We start at -1 because we increment it before we use it
	for i := uint64(0); i < shards; i++ {
		if uint64(len(shardArr)) >= perCluster {
			if cid >= len(clusterNames)-3 {
				// Create a new cluster name using random string
				clusterNames = append(clusterNames, coreutils.RandomString(10))
			}
			cid++
			clusterMap = append(clusterMap, proc.ClusterMap{ID: cid, Name: clusterNames[cid], Shards: shardArr})
			shardArr = []uint64{}
		}

		shardArr = append(shardArr, i)
	}

	if len(shardArr) > 0 {
		cid++
		clusterMap = append(clusterMap, proc.ClusterMap{ID: cid, Name: clusterNames[cid], Shards: shardArr})
	}

	return clusterMap
}
