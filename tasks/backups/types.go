package backups

import (
	"bytes"

	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/iblfile"
)

const (
	totalMaxMessages = 10000
	fileType         = "backup.server"
)

type ChannelMsgCount struct {
	PerChannel         int            `json:"per_channel" description:"The number of messages per channel"`
	BackupAttachments  bool           `json:"backup_attachments" description:"Whether to backup attachments or not"`
	SpecialAllocations map[string]int `json:"special_allocations" description:"Specific channel allocation overrides"`
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
