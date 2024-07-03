package types

import (
	"github.com/anti-raid/splashtail/core/go.std/silverpelt"
	orderedmap "github.com/wk8/go-ordered-map/v2"
)

// PatchGuildModuleConfiguration allows updating the guild module configuration
type PatchGuildModuleConfiguration struct {
	Module       string                                  `json:"module" description:"The module to update"`
	Disabled     *Clearable[bool]                        `json:"disabled,omitempty" description:"Whether or not the module is disabled or not. If null, use default for module"`                   // Whether or not the module is disabled or not. None means to use the default module configuration
	DefaultPerms *Clearable[silverpelt.PermissionChecks] `json:"default_perms,omitempty" description:"The default permission checks of the module, can be overrided by the command configuration"` // The default permission checks of the module, can be overrided by the command configuration
}

// PatchGuildModuleConfiguration allows updating the guild module configuration
type SettingsExecute struct {
	Operation silverpelt.CanonicalOperationType  `json:"operation" description:"The operation type to execute"`
	Module    string                             `json:"module" description:"The module in which the setting is in"`
	Setting   string                             `json:"setting" description:"The name of the setting"`
	Fields    orderedmap.OrderedMap[string, any] `json:"fields" description:"The fields to execute the operation with"`
}
