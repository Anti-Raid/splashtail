package types

// PatchGuildModuleConfiguration allows updating the guild module configuration
type PatchGuildModuleConfiguration struct {
	Disabled *bool `db:"disabled" json:"disabled,omitempty" description:"Whether or not the module is disabled or not. If null, use default for module"` // Whether or not the module is disabled or not. None means to use the default module configuration
}
