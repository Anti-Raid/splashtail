package core

import (
	"catway/core/sandwichjson"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"os"
	"sync"

	"github.com/disgoorg/disgo"
	"github.com/disgoorg/disgo/bot"
	"github.com/disgoorg/disgo/cache"
	"github.com/disgoorg/disgo/events"
	"github.com/disgoorg/disgo/gateway"
	"github.com/disgoorg/disgo/sharding"
	"github.com/disgoorg/snowflake/v2"
)

type void struct{}

type CatwayConfig struct {
	Token   string          `json:"token" yaml:"token"`
	Intents gateway.Intents `json:"intents" yaml:"intents"`

	VirtualShards struct {
		Enabled bool  `json:"enabled" yaml:"enabled"`
		Count   int32 `json:"count" yaml:"count"`       // Number of virtual shards to use
		DmShard int32 `json:"dm_shard" yaml:"dm_shard"` // Shard to use for DMs and non identifiable events
	} `json:"virtual_shards" yaml:"virtual_shards"`

	Rest struct {
		GetGatewayBot struct {
			MaxConcurrency int32 `json:"max_concurrency" yaml:"max_concurrency"` // How many requests to allow in the custom get gateway bot impl
		}
	}

	Sharding struct {
		ShardIDs   []int `json:"shard_ids" yaml:"shard_ids"`     // Shard IDs to use
		ShardCount int   `json:"shard_count" yaml:"shard_count"` // Number of shards to use
	}

	Producer struct {
		Configuration map[string]interface{} `json:"configuration" yaml:"configuration"`
		Type          string                 `json:"type" yaml:"type"`
	} `json:"producer" yaml:"producer"`

	Messaging struct {
		ClientName      string `json:"client_name" yaml:"client_name"`
		ChannelName     string `json:"channel_name" yaml:"channel_name"`
		UseRandomSuffix bool   `json:"use_random_suffix" yaml:"use_random_suffix"`
	} `json:"messaging" yaml:"messaging"`

	CacheFlags *cache.Flags `json:"cache_flags" yaml:"cache_flags"`
}

type Catway struct {
	configurationMu sync.RWMutex
	Config          *CatwayConfig

	ProducerClient MQClient `json:"-"`
	Discord        bot.Client
	Logger         *slog.Logger
	Context        context.Context
	ContextClose   context.CancelFunc
}

func NewCatway(cfg *CatwayConfig) (*Catway, error) {
	ctx, cancel := context.WithCancel(context.Background())

	// Setup sharding options
	var shardingOpts = []sharding.ConfigOpt{
		sharding.WithGatewayConfigOpts(
			gateway.WithIntents(cfg.Intents),
			gateway.WithCompress(true),        // Always compress
			gateway.WithEnableRawEvents(true), // Always enable raw events as this is required
		),
	}

	if len(cfg.Sharding.ShardIDs) > 0 {
		shardingOpts = append(shardingOpts, sharding.WithShardIDs(cfg.Sharding.ShardIDs...))
	}

	if cfg.Sharding.ShardCount > 0 {
		shardingOpts = append(shardingOpts, sharding.WithShardCount(cfg.Sharding.ShardCount))
	}

	// Set cache flags, default to all if unset
	var cacheFlags cache.Flags

	if cfg.CacheFlags != nil {
		cacheFlags = *cfg.CacheFlags
	} else {
		cacheFlags = cache.FlagsAll
	}

	var catway *Catway
	discord, err := disgo.New(cfg.Token, bot.WithShardManagerConfigOpts(
		shardingOpts...,
	),
		bot.WithCacheConfigOpts(
			cache.WithCaches(cacheFlags),
		),
		bot.WithEventListeners(&events.ListenerAdapter{
			OnRaw: func(event *events.Raw) {
				if event.EventType == gateway.EventTypeReady ||
					event.EventType == gateway.EventTypeResumed ||
					event.EventType == gateway.EventTypeHeartbeatAck {
					return
				}

				// Get the event dispatch identifier
				identifier := GetEventDispatcher(event)

				data, err := io.ReadAll(event.Payload)

				if err != nil {
					slog.Error("Failed to read payload", slog.String("err", err.Error()))
					return
				}

				// Create the payload
				payload := &CatwayPayload{
					Metadata: &CatwayMetadata{
						Version:    "1",
						Identifier: "catway",
						Shard:      [2]int32{int32(event.ShardID()), int32(catway.ShardCount())},
					},
					Type:                    string(event.EventType),
					Data:                    json.RawMessage(data),
					Sequence:                event.SequenceNumber(),
					Op:                      gateway.OpcodeDispatch,
					EventDispatchIdentifier: identifier,
				}

				// Publish the payload
				if err := PublishEvent(ctx, catway, payload); err != nil {
					slog.Error("Failed to publish event", slog.String("err", err.Error()))
				}
			},
			OnGuildsReady: func(event *events.GuildsReady) {
				slog.Info("All guilds ready", slog.Int("shardId", event.ShardID()))
			},
			OnReady: func(event *events.Ready) {
				slog.Info("Bot ready", slog.Int("shardId", event.ShardID()))
			},
		}),
	)

	if err != nil {
		cancel()
		return nil, err
	}

	producerClient, err := NewMQClient(cfg.Producer.Type)
	if err != nil {
		cancel()
		return nil, err
	}

	clientName := cfg.Messaging.ClientName
	if cfg.Messaging.UseRandomSuffix {
		clientName = clientName + "-" + randomHex(6)
	}

	catway = &Catway{
		Config: cfg,

		ProducerClient: producerClient,
		Discord:        discord,
		Logger:         slog.New(slog.NewTextHandler(os.Stderr, nil)),
		Context:        ctx,
		ContextClose:   cancel,
	}

	go func() {
		defer func() {
			if err := recover(); err != nil {
				catway.Logger.Error("Recovered from panic", slog.Any("err", err))
			}
		}()

		if err := discord.OpenShardManager(ctx); err != nil {
			catway.Logger.Error("Failed to connect to discord", slog.String("err", err.Error()))
		}

		catway.Logger.Info("Connecting to discord...")
	}()

	err = producerClient.Connect(
		ctx,
		catway,
		clientName,
		cfg.Producer.Configuration,
	)

	if err != nil {
		cancel()
		return catway, fmt.Errorf("failed to connect to producer: %w", err)
	}

	return catway, nil
}

func (c *Catway) Close() error {
	c.ProducerClient.Close()
	c.Discord.Close(context.Background())
	c.ContextClose()
	return nil
}

func (c *Catway) ShardCount() int32 {
	return int32(len(c.Discord.ShardManager().Shards()))
}

// GetEventDispatcher returns the event dispatcher given a events.Raw
func GetEventDispatcher(payload *events.Raw) *EventDispatchIdentifier {
	switch payload.EventType {
	case gateway.EventTypeGuildCreate:
	case gateway.EventTypeGuildUpdate:
	case gateway.EventTypeGuildDelete:
		var guildCreatePayload struct {
			ID snowflake.ID `json:"id"`
		}

		if err := sandwichjson.UnmarshalReader(payload.Payload, &guildCreatePayload); err == nil {
			return &EventDispatchIdentifier{
				GuildID: &guildCreatePayload.ID,
			}
		}
	case gateway.EventTypeUserUpdate:
		return &EventDispatchIdentifier{
			GloballyRouted: true, // No Guild ID is available for routing *user* (not member) updates, send to all shards
		}
	default:
		var defaultPayload struct {
			GuildID *snowflake.ID `json:"guild_id,omitempty"`
		}

		if err := sandwichjson.UnmarshalReader(payload.Payload, &defaultPayload); err == nil {
			return &EventDispatchIdentifier{
				GuildID:        defaultPayload.GuildID,
				GloballyRouted: defaultPayload.GuildID == nil,
			}
		}
	}

	return &EventDispatchIdentifier{}
}

// ConsumerShardCount returns the number of shards from a consumer view
//
// If virtual shards is disabled, this will return the actual shard count.
// If virtual shards is enabled, this will return the virtual shard count.
func (mg *Catway) ConsumerShardCount() int32 {
	if mg.Config.VirtualShards.Enabled {
		return mg.Config.VirtualShards.Count
	}

	return int32(mg.ShardCount())
}

// GetShardIdOfGuild returns the shard id of a guild
func (mg *Catway) GetShardIdOfGuild(guildID snowflake.ID, shardCount int32) int32 {
	if shardCount <= 0 {
		mg.Logger.Error("Shard count is 0, cannot calculate shard id")
		return 0
	}

	return int32((int64(guildID) >> 22) % int64(shardCount))
}

// RoutePayloadToConsumer routes a SandwichPayload to its corresponding consumer modifying the payload itself
func (c *Catway) RoutePayloadToConsumer(payload *CatwayPayload) error {
	if !c.Config.VirtualShards.Enabled {
		// No need to remap, return
		return nil
	}

	if payload.EventDispatchIdentifier == nil {
		return fmt.Errorf("eventDispatchIdentifier is nil and cannot be remapped")
	}

	if payload.EventDispatchIdentifier.GloballyRouted {
		// Remap shard to empty
		payload.Metadata.Shard = [2]int32{}
	} else if payload.EventDispatchIdentifier.GuildID != nil && *payload.EventDispatchIdentifier.GuildID != 0 {
		virtualShardId := c.GetShardIdOfGuild(*payload.EventDispatchIdentifier.GuildID, c.Config.VirtualShards.Count)
		payload.Metadata.Shard = [2]int32{virtualShardId, c.Config.VirtualShards.Count}
	} else {
		// Not globally routed + no guild id means it's a DM
		payload.Metadata.Shard = [2]int32{c.Config.VirtualShards.DmShard, c.Config.VirtualShards.Count}
	}

	return nil
}

// Used as a key for virtual shard dispatches etc., must be set for all events
type EventDispatchIdentifier struct {
	GuildID        *snowflake.ID
	GloballyRouted bool // Whether or not the event should be globally routed
}

// SandwichMetadata represents the identification information that consumers will use.
type CatwayMetadata struct {
	Version    string `json:"v"`
	Identifier string `json:"i"`
	// Shard ID, Shard Count
	Shard [2]int32 `json:"s"`
}

type CatwayPayload struct {
	Metadata *CatwayMetadata `json:"__metadata"`
	Type     string          `json:"t"`

	Data                    json.RawMessage          `json:"d"`
	Sequence                int                      `json:"s"`
	Op                      gateway.Opcode           `json:"op"`
	EventDispatchIdentifier *EventDispatchIdentifier `json:"-"`
}
