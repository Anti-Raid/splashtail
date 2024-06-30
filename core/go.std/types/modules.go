package types

import "github.com/anti-raid/splashtail/core/go.std/silverpelt"

// PatchGuildModuleConfiguration allows updating the guild module configuration
type PatchGuildModuleConfiguration struct {
	Module       string                                  `json:"module" description:"The module to update"`
	Disabled     *Clearable[bool]                        `json:"disabled,omitempty" description:"Whether or not the module is disabled or not. If null, use default for module"`                   // Whether or not the module is disabled or not. None means to use the default module configuration
	DefaultPerms *Clearable[silverpelt.PermissionChecks] `json:"default_perms,omitempty" description:"The default permission checks of the module, can be overrided by the command configuration"` // The default permission checks of the module, can be overrided by the command configuration
}
