// From silverpelt/mod.rs
package silverpelt

import (
	"strings"
)

type NativePermission string

func (np NativePermission) String() string {
	return string(np) // TODO: Support discord permissions
}

// PermissionCheck represents the permissions needed to run a command.
type PermissionCheck struct {
	KittycatPerms []string           `json:"kittycat_perms"` // The kittycat permissions needed to run the command
	NativePerms   []NativePermission `json:"native_perms"`   // The native permissions needed to run the command (converted from serenity::all::Permissions)
	OuterAnd      bool               `json:"outer_and"`      // Whether the next permission check should be ANDed (all needed) or OR'd (at least one) to the current
	InnerAnd      bool               `json:"inner_and"`      // Whether or not the perms are ANDed (all needed) or OR'd (at least one)
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

// PermissionChecks represents a list of permission checks.
type PermissionChecks struct {
	Checks       []PermissionCheck `json:"checks"`        // The list of permission checks
	ChecksNeeded int               `json:"checks_needed"` // Number of checks that need to be true
}

func (pcs PermissionChecks) String() string {
	var sb strings.Builder
	for i, check := range pcs.Checks {
		if i != 0 {
			sb.WriteString(" ")
		}
		sb.WriteString(check.String())
		empty := len(check.KittycatPerms) == 0 && len(check.NativePerms) == 0
		if i < len(pcs.Checks)-1 {
			if check.OuterAnd && !empty {
				sb.WriteString("AND ")
			} else {
				sb.WriteString("OR ")
			}
		}
	}
	return sb.String()
}

// CommandExtendedData represents the default permissions needed to run a command.
type CommandExtendedData struct {
	DefaultPerms     PermissionChecks `json:"default_perms"`      // The default permissions needed to run this command
	IsDefaultEnabled bool             `json:"is_default_enabled"` // Whether or not the command is enabled by default
}

// NewCommandExtendedData creates a new CommandExtendedData with default values.
func NewCommandExtendedData() CommandExtendedData {
	return CommandExtendedData{
		DefaultPerms: PermissionChecks{
			Checks:       []PermissionCheck{},
			ChecksNeeded: 0,
		},
		IsDefaultEnabled: true,
	}
}

// GuildCommandConfiguration represents guild command configuration data.
type GuildCommandConfiguration struct {
	ID       string            `db:"id" json:"id" description:"ID of the command configuration entry"`                                                           // The ID
	GuildID  string            `db:"guild_id" json:"guild_id" description:"Guild ID the command configuration entry pertains to"`                                // The guild id (from db)
	Command  string            `db:"command" json:"command" description:"The name of the command"`                                                               // The command name
	Perms    *PermissionChecks `db:"perms" json:"commands" description:"Any custom permission settings"`                                                         // The permission method (kittycat)
	Disabled *bool             `db:"disabled" json:"disabled,omitempty" description:"Whether the command is disabled or not.  If null, use default for command"` // Whether or not the command is disabled
}

// GuildModuleConfiguration represents guild module configuration data.
type GuildModuleConfiguration struct {
	ID       string `db:"id" json:"id" description:"ID of the command configuration entry"`                                                               // The ID
	GuildID  string `db:"guild_id" json:"guild_id" description:"Guild ID the module configuration entry pertains to"`                                     // The guild id (from db)
	Module   string `db:"module" json:"module" description:"The module's name ('id')"`                                                                    // The module id
	Disabled *bool  `db:"disabled" json:"disabled,omitempty" description:"Whether or not the module is disabled or not. If null, use default for module"` // Whether or not the module is disabled or not. None means to use the default module configuration
}
