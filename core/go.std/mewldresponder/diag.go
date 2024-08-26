package mewldresponder

type MewldDiagPayload struct {
	ClusterID uint16 `json:"id"`
	Nonce     string `json:"nonce"`
	Diag      bool   `json:"diag"`
}

type MewldDiagResponse struct {
	ClusterID uint16                 `json:"cluster_id"`
	Nonce     string                 `json:"Nonce"`
	Data      []MewldDiagShardHealth `json:"Data"`
}

type MewldDiagShardHealth struct {
	ShardID uint16  `json:"shard_id"`
	Up      bool    `json:"up"`
	Latency float64 `json:"latency"`
	Guilds  uint64  `json:"guilds"`
	Users   uint64  `json:"users"`
}
