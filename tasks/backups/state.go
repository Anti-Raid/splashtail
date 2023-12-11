package backups

import "splashtail/syncmap"

// concurrentBackupState is a map of guild IDs to the number of backup tasks
// they have running concurrently.
var concurrentBackupState = syncmap.Map[string, int]{} // guildID -> concurrent tasks
