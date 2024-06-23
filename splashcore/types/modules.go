package types

import "github.com/anti-raid/splashtail/splashcore/silverpelt"

// PatchGuildModuleConfiguration allows updating the guild module configuration
type PatchGuildModuleConfiguration struct {
	Disabled     *bool                        `db:"disabled" json:"disabled,omitempty" description:"Whether or not the module is disabled or not. If null, use default for module"`                        // Whether or not the module is disabled or not. None means to use the default module configuration
	DefaultPerms *silverpelt.PermissionChecks `db:"default_perms" json:"default_perms,omitempty" description:"The default permission checks of the module, can be overrided by the command configuration"` // The default permission checks of the module, can be overrided by the command configuration
}
