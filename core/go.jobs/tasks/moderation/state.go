package moderation

import "go.std/utils/syncmap"

// concurrentModerationState is a map of guild IDs to the number of moderation tasks
// they have running concurrently.
var concurrentModerationState = syncmap.Map[string, int]{} // guildID -> concurrent tasks
