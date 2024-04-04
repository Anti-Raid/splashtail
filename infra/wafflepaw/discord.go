package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

type discordGetGatewayBot struct {
	Shards uint64 `json:"shards"`
}

func GetDiscordGetGatewayBot() (*discordGetGatewayBot, error) {
	req, err := http.NewRequest("GET", Config.Meta.Proxy+"/api/v10/gateway/bot", nil)
	//req, err := http.NewRequest("GET", "https://discord.com/api/v10/gateway/bot", nil)

	if err != nil {
		return nil, err
	}

	req.Header.Set("Authorization", "Bot "+Config.DiscordAuth.Token)

	client := http.Client{
		Timeout: 5 * time.Second,
	}

	resp, err := client.Do(req)

	if err != nil {
		return nil, err
	}

	if resp.StatusCode != 200 {
		text, err := io.ReadAll(resp.Body)

		if err != nil {
			return nil, fmt.Errorf("non 200 status code: %d", resp.StatusCode)
		}

		return nil, fmt.Errorf("non 200 status code: %d, %s", resp.StatusCode, text)
	}

	defer resp.Body.Close()

	var data discordGetGatewayBot

	err = json.NewDecoder(resp.Body).Decode(&data)

	if err != nil {
		return nil, err
	}

	return &data, nil
}
