package backups

import (
	"bytes"

	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/iblfile"
)

const (
	totalMaxMessages         = 500
	maxAttachmentFileSize    = 8_000_000  // 8 MB, the limit for one attachment
	fileSizeWarningThreshold = 50_000_000 // 50 MB, the warning threshold for the total file size. At this point, attachments will not be saved
	minPerChannel            = 50
	defaultPerChannel        = 100
	jpegReencodeQuality      = 75
	fileType                 = "backup.server"
	restoreMaxBodySize       = 100_000_000 // 100 MB, the maximum size of the backup file
)

var allowedChannelTypes = []discordgo.ChannelType{
	discordgo.ChannelTypeGuildText,
	discordgo.ChannelTypeGuildNews,
	discordgo.ChannelTypeGuildNewsThread,
	discordgo.ChannelTypeGuildPublicThread,
	discordgo.ChannelTypeGuildPrivateThread,
	discordgo.ChannelTypeGuildForum,
}

type AttachmentStorageFormat string

const (
	AttachmentStorageFormatUnknownOrUnsaved AttachmentStorageFormat = ""
	AttachmentStorageFormatUncompressed     AttachmentStorageFormat = "uncompressed"
	AttachmentStorageFormatGzip             AttachmentStorageFormat = "gzip"
	AttachmentStorageFormatJpegEncoded      AttachmentStorageFormat = "jpeg_encoded"
	AttachmentStorageFormatRemote           AttachmentStorageFormat = "remote"
)

// Options that can be set when creatng a backup
type BackupCreateOpts struct {
	PerChannel                int            `json:"per_channel" description:"The number of messages per channel"`
	MaxMessages               int            `json:"max_messages" description:"The maximum number of messages to backup"`
	BackupMessages            bool           `json:"backup_messages" description:"Whether to backup messages or not"`
	BackupAttachments         bool           `json:"backup_attachments" description:"Whether to backup attachments or not"`
	IgnoreMessageBackupErrors bool           `json:"ignore_message_backup_errors" description:"Whether to ignore errors while backing up messages or not and skip these channels"`
	RolloverLeftovers         bool           `json:"rollover_leftovers" description:"Whether to attempt rollover of leftover message quota to another channels or not"`
	SpecialAllocations        map[string]int `json:"special_allocations" description:"Specific channel allocation overrides"`
	Encrypt                   string         `json:"encrypt" description:"The key to encrypt backups with, if any"`
}

// Options that can be set when restoring a backup
type BackupRestoreOpts struct {
	ProtectedChannels []string `json:"protected_channels" description:"Channels to protect from being deleted"`
	BackupSource      string   `json:"backup_source" description:"The source of the backup"`
	Decrypt           string   `json:"decrypt" description:"The key to decrypt backups with, if any"`
}

// Attachment contains metadata about an attachment
type AttachmentMetadata struct {
	ID            string                  `json:"id"`             // ID of the attachment within the ticket
	URL           string                  `json:"url"`            // URL of the attachment
	ProxyURL      string                  `json:"proxy_url"`      // URL (cached) of the attachment
	Name          string                  `json:"name"`           // Name of the attachment
	ContentType   string                  `json:"content_type"`   // Content type of the attachment
	StorageFormat AttachmentStorageFormat `json:"storage_format"` // Storage format of the attachment
	Size          int                     `json:"size"`           // Size of the attachment in bytes
	Errors        []string                `json:"errors"`         // Non-fatal errors that occurred while uploading the attachment
}

// Represents a backed up message
type BackupMessage struct {
	Message            *discordgo.Message       `json:"message"`
	AttachmentMetadata []AttachmentMetadata     `json:"attachment_metadata"`
	attachments        map[string]*bytes.Buffer `json:"-"`
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
