package backups

import (
	"bytes"

	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/iblfile"
)

const (
	totalMaxMessages = 500
	fileType         = "backup.server"
)

type BackupOpts struct {
	PerChannel                int            `json:"per_channel" description:"The number of messages per channel"`
	MaxMessages               int            `json:"max_messages" description:"The maximum number of messages to backup"`
	BackupMessages            bool           `json:"backup_messages" description:"Whether to backup messages or not"`
	BackupAttachments         bool           `json:"backup_attachments" description:"Whether to backup attachments or not"`
	IgnoreMessageBackupErrors bool           `json:"ignore_message_backup_errors" description:"Whether to ignore errors while backing up messages or not and skip these channels"`
	RolloverLeftovers         bool           `json:"rollover_leftovers" description:"Whether to attempt rollover of leftover message quota to another channels or not"`
	SpecialAllocations        map[string]int `json:"special_allocations" description:"Specific channel allocation overrides"`
	Encrypt                   bool           `json:"encrypt" description:"Whether to encrypt the backup or not"`
}

type CoreBackup struct {
	Guild *discordgo.Guild `db:"guild" json:"guild" description:"The guild ID"`
}

func init() {
	iblfile.RegisterFormat("backup", &iblfile.Format{
		Format:  "server",
		Version: "a1",
		GetExtended: func(section map[string]*bytes.Buffer, meta *iblfile.Meta) (map[string]any, error) {
			return map[string]any{}, nil
		},
	})
}
