package core

import (
	"context"
	"fmt"
)

type MQClient interface {
	String() string
	Channel() string

	Connect(ctx context.Context, catway *Catway, clientName string, args map[string]interface{}) error
	Publish(ctx context.Context, packet *CatwayPayload, channel string) error

	// IsClosed returns true if the connection is closed.
	IsClosed() bool

	// Close all connections for a specific shard
	CloseShard(shardID int32)

	// Close the connection
	Close()
}

func NewMQClient(mqType string) (MQClient, error) {
	switch mqType {
	case "websocket":
		return &WebsocketClient{}, nil
	default:
		return nil, fmt.Errorf("%s is not a valid MQClient", mqType)
	}
}

// PublishEvent publishes a SandwichPayload.
func PublishEvent(ctx context.Context, c *Catway, packet *CatwayPayload) error {
	c.configurationMu.RLock()
	channelName := c.Config.Messaging.ChannelName
	c.configurationMu.RUnlock()

	err := c.RoutePayloadToConsumer(packet)

	if err != nil {
		return fmt.Errorf("publishEvent RoutePayloadToConsumer: %w", err)
	}

	err = c.ProducerClient.Publish(
		ctx,
		packet,
		channelName,
	)
	if err != nil {
		return fmt.Errorf("publishEvent publish: %w", err)
	}

	return nil
}
