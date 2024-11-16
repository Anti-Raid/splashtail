// From silverpelt/mod.rs
package silverpelt

import (
	"context"
	"strings"
	"time"

	"go.std/bigint"
)

// PermissionCheck represents the permissions needed to run a command.
type PermissionCheck struct {
	KittycatPerms []string        `json:"kittycat_perms"`              // The kittycat permissions needed to run the command
	NativePerms   []bigint.BigInt `json:"native_perms" type:"integer"` // The native permissions needed to run the command (converted from serenity::all::Permissions)
	InnerAnd      bool            `json:"inner_and"`                   // Whether or not the perms are ANDed (all needed) or OR'd (at least one)
}

func (pc PermissionCheck) String() string {
	var sb strings.Builder
	if len(pc.NativePerms) > 0 {
		sb.WriteString("\nDiscord: ")
		for j, perm := range pc.NativePerms {
			if j != 0 {
				sb.WriteString(" ")
			}
			sb.WriteString(perm.String())
			if j < len(pc.NativePerms)-1 {
				if pc.InnerAnd {
					sb.WriteString(" AND")
				} else {
					sb.WriteString(" OR")
				}
			}
		}
	}
	if len(pc.KittycatPerms) > 0 {
		sb.WriteString("\nCustom Permissions (kittycat): ")
		for j, perm := range pc.KittycatPerms {
			if j != 0 {
				sb.WriteString(" ")
			}
			sb.WriteString(perm)
			if j < len(pc.KittycatPerms)-1 {
				if pc.InnerAnd {
					sb.WriteString(" AND")
				} else {
					sb.WriteString(" OR")
				}
			}
		}
	}
	return sb.String()
}

// CommandExtendedData represents the default permissions needed to run a command.
type CommandExtendedData struct {
	DefaultPerms     PermissionCheck `json:"default_perms"`      // The default permissions needed to run this command
	IsDefaultEnabled bool            `json:"is_default_enabled"` // Whether or not the command is enabled by default
	WebHidden        bool            `json:"web_hidden"`         // Whether or not the command is hidden from the web interface
	VirtualCommand   bool            `json:"virtual_command"`    // Whether or not the command is a virtual command or not
}

// GuildCommandConfiguration represents guild command configuration data.
type GuildCommandConfiguration struct {
	ID       string           `db:"id" json:"id" description:"ID of the command configuration entry"`                                                                                                                                   // The ID
	GuildID  string           `db:"guild_id" json:"guild_id" description:"Guild ID the command configuration entry pertains to"`                                                                                                        // The guild id (from db)
	Command  string           `db:"command" json:"command" description:"The name of the command"`                                                                                                                                       // The command name
	Perms    *PermissionCheck `db:"perms" json:"perms" description:"The permission checks on the command, if unset, will revert to either the modules default_perms and if that is unset, the default perms set on the command itself"` // The permission checks on the command, if unset, will revert to either the modules default_perms and if that is unset, the default perms set on the command itself
	Disabled *bool            `db:"disabled" json:"disabled,omitempty" description:"Whether the command is disabled or not.  If null, use default for command"`                                                                         // Whether or not the command is disabled
}

func (gcc GuildCommandConfiguration) ToFullGuildCommandConfiguration(ctx context.Context, c DbConn) (*FullGuildCommandConfiguration, error) {
	var createdAt time.Time
	var createdBy string
	var lastUpdatedAt time.Time
	var lastUpdatedBy string

	err := c.QueryRow(ctx, "SELECT created_at, created_by, last_updated_at, last_updated_by FROM guild_command_configurations WHERE id = $1", gcc.ID).Scan(&createdAt, &createdBy, &lastUpdatedAt, &lastUpdatedBy)

	if err != nil {
		return nil, err
	}

	return &FullGuildCommandConfiguration{
		ID:            gcc.ID,
		GuildID:       gcc.GuildID,
		Command:       gcc.Command,
		Perms:         gcc.Perms,
		Disabled:      gcc.Disabled,
		CreatedAt:     createdAt,
		CreatedBy:     createdBy,
		LastUpdatedAt: lastUpdatedAt,
		LastUpdatedBy: lastUpdatedBy,
	}, nil
}

// FullGuildCommandConfiguration represents the full guild command configuration data including audit info etc.
type FullGuildCommandConfiguration struct {
	ID            string           `db:"id" json:"id" description:"ID of the command configuration entry"`                                                                                                                                   // The ID
	GuildID       string           `db:"guild_id" json:"guild_id" description:"Guild ID the command configuration entry pertains to"`                                                                                                        // The guild id (from db)
	Command       string           `db:"command" json:"command" description:"The name of the command"`                                                                                                                                       // The command name
	Perms         *PermissionCheck `db:"perms" json:"perms" description:"The permission checks on the command, if unset, will revert to either the modules default_perms and if that is unset, the default perms set on the command itself"` // The permission checks on the command, if unset, will revert to either the modules default_perms and if that is unset, the default perms set on the command itself
	Disabled      *bool            `db:"disabled" json:"disabled,omitempty" description:"Whether the command is disabled or not.  If null, use default for command"`                                                                         // Whether or not the command is disabled
	CreatedAt     time.Time        `db:"created_at" json:"created_at" description:"The time the command configuration was created"`                                                                                                          // The time the command configuration was created
	CreatedBy     string           `db:"created_by" json:"created_by" description:"The user who created the command configuration"`                                                                                                          // The user who created the command configuration
	LastUpdatedAt time.Time        `db:"last_updated_at" json:"last_updated_at" description:"The time the command configuration was last updated"`                                                                                           // The time the command configuration was last updated
	LastUpdatedBy string           `db:"last_updated_by" json:"last_updated_by" description:"The user who last updated the command configuration"`                                                                                           // The user who last updated the command configuration
}

func (f *FullGuildCommandConfiguration) ToGuildCommandConfiguration() *GuildCommandConfiguration {
	return &GuildCommandConfiguration{
		ID:       f.ID,
		GuildID:  f.GuildID,
		Command:  f.Command,
		Perms:    f.Perms,
		Disabled: f.Disabled,
	}
}

// GuildModuleConfiguration represents guild module configuration data.
type GuildModuleConfiguration struct {
	ID       string `db:"id" json:"id" description:"ID of the command configuration entry"`                                                               // The ID
	GuildID  string `db:"guild_id" json:"guild_id" description:"Guild ID the module configuration entry pertains to"`                                     // The guild id (from db)
	Module   string `db:"module" json:"module" description:"The module's name ('id')"`                                                                    // The module id
	Disabled *bool  `db:"disabled" json:"disabled,omitempty" description:"Whether or not the module is disabled or not. If null, use default for module"` // Whether or not the module is disabled or not. None means to use the default module configuration
}

func (gmc GuildModuleConfiguration) Fill() *GuildModuleConfiguration {
	// Should be changed/expanded later
	return &gmc
}
