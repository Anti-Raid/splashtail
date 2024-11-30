package types

import "github.com/Anti-Raid/corelib_go/silverpelt"

// PatchGuildModuleConfiguration allows updating the guild module configuration
type PatchGuildCommandConfiguration struct {
	Command  string                                 `json:"command" description:"The command to update"`
	Disabled *Clearable[bool]                       `json:"disabled,omitempty" description:"Whether or not the command is disabled or not. If null, use default for module"`  // Whether or not the module is disabled or not. None means to use the default module configuration
	Perms    *Clearable[silverpelt.PermissionCheck] `json:"perms,omitempty" description:"The permission check of the command, can be overrided by the command configuration"` // The default permission check of the command, can be overrided by the command configuration
}
