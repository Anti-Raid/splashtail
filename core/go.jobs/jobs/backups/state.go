package backups

import "go.std/utils/syncmap"

// concurrentBackupState is a map of guild IDs to the number of backup-related jobs
// they have running concurrently.
var concurrentBackupState = syncmap.Map[string, int]{} // guildID -> concurrent jobs
