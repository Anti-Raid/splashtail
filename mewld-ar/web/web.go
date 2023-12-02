package web

import (
	"encoding/json"
	"io"
	"mewld/config"
	"net/http"
	"time"

	log "github.com/sirupsen/logrus"
)

type SessionStartLimit struct {
	Total          uint64 `json:"total"`
	Remaining      uint64 `json:"remaining"`
	ResetAfter     uint64 `json:"reset_after"`
	MaxConcurrency uint64 `json:"max_concurrency"`
}

type ShardCount struct {
	Shards            uint64            `json:"shards"`
	SessionStartLimit SessionStartLimit `json:"session_start_limit"`
}

func GetShardCount(config config.CoreConfig) ShardCount {
	url := "https://discord.com/api/gateway/bot"

	req, err := http.NewRequest("GET", url, nil)

	req.Header.Add("Authorization", "Bot "+config.Token)
	req.Header.Add("User-Agent", "DiscordBot Antiraid/6.0 (mewld)") // ANTIRAID-SPECIFIC: Change user agent
	req.Header.Add("Content-Type", "application/json")

	if err != nil {
		log.Fatal(err)
	}

	client := http.Client{Timeout: 10 * time.Second}

	res, err := client.Do(req)

	if err != nil {
		log.Fatal(err)
	}

	defer res.Body.Close()

	log.Println("Shard count status:", res.Status)

	if res.StatusCode != 200 {
		log.Fatal("Shard count status code not 200. Invalid token?")
	}

	var shardCount ShardCount

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		log.Fatal(err)
	}

	err = json.Unmarshal(bodyBytes, &shardCount)

	if err != nil {
		log.Fatal(err)
	}

	if shardCount.SessionStartLimit.Remaining < 10 {
		log.Fatal("Shard count remaining is less than safe value of 10")
	}

	return shardCount
}
