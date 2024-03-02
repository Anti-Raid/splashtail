package moderation

// Options that can be set when pruning a message
type MsgPruneOpts struct {
	UserID       string   `description:"If set, the user id to prune messages of"`
	Channels     []string `description:"If set, the channels to prune messages from"`
	IgnoreErrors bool     `description:"If set, ignore errors while pruning"`
}

type MsgPruneConstraints struct {
	MaxMessages int `description:"The maximum number of messages to prune"`
}

type ModerationConstraints struct {
	MsgPrune                 *MsgPruneConstraints
	MaxServerModerationTasks int // How many moderation tasks can run concurrently per server
}

var FreePlanBackupConstraints = &ModerationConstraints{
	MsgPrune: &MsgPruneConstraints{
		MaxMessages: 1000,
	},
	MaxServerModerationTasks: 5,
}
