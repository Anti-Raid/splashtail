package backups

import "github.com/anti-raid/splashtail/syncmap"

// concurrentBackupState is a map of guild IDs to the number of backup tasks
// they have running concurrently.
var concurrentBackupState = syncmap.Map[string, int]{} // guildID -> concurrent tasks
