package moderation

import "go.std/utils/timex"

// Options that can be set when pruning a message
type MessagePruneOpts struct {
	UserID             string         `description:"If set, the user id to prune messages of"`
	Channels           []string       `description:"If set, the channels to prune messages from"`
	IgnoreErrors       bool           `description:"If set, ignore errors while pruning"`
	MaxMessages        int            `description:"The maximum number of messages to prune"`
	PruneFrom          timex.Duration `description:"If set, the time to prune messages from."`
	PerChannel         int            `description:"The minimum number of messages to prune per channel"`
	RolloverLeftovers  bool           `description:"Whether to attempt rollover of leftover message quota to another channels or not"`
	SpecialAllocations map[string]int `description:"Specific channel allocation overrides"`
}

type MessagePruneConstraints struct {
	TotalMaxMessages int `description:"The maximum number of messages to prune"`
	MinPerChannel    int `description:"The minimum number of messages to prune per channel"`
}

type ModerationConstraints struct {
	MessagePrune        *MessagePruneConstraints
	MaxServerModeration int // How many moderation related jobs can run concurrently per server
}

var FreePlanModerationConstraints = &ModerationConstraints{
	MessagePrune: &MessagePruneConstraints{
		TotalMaxMessages: 1000,
		MinPerChannel:    10,
	},
	MaxServerModeration: 5,
}
