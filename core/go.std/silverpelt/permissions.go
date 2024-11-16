// From botv2 silverpelt/permissions.rs
package silverpelt

import "fmt"

type PermissionResult struct {
	Var           string                     `json:"var"`
	Message       string                     `json:"message,omitempty"`
	Check         *PermissionCheck           `json:"check,omitempty"`
	CommandConfig *GuildCommandConfiguration `json:"command_config,omitempty"`
	ModuleConfig  *GuildModuleConfiguration  `json:"module_config,omitempty"`
	Error         string                     `json:"error,omitempty"`
}

func NewPermissionResultFromError[T fmt.Stringer](e T) PermissionResult {
	return PermissionResult{
		Var:   "GenericError",
		Error: e.String(),
	}
}

func (p PermissionResult) Code() string {
	switch p.Var {
	case "Ok":
		return "ok"
	case "OkWithMessage":
		return "ok_with_message"
	case "MissingKittycatPerms":
		return "missing_kittycat_perms"
	case "MissingNativePerms":
		return "missing_native_perms"
	case "MissingAnyPerms":
		return "missing_any_perms"
	case "CommandDisabled":
		return "command_disabled"
	case "UnknownModule":
		return "unknown_module"
	case "ModuleNotFound":
		return "module_not_found"
	case "ModuleDisabled":
		return "module_disabled"
	case "DiscordError":
		return "discord_error"
	case "SudoNotGranted":
		return "sudo_not_granted"
	case "GenericError":
		return "generic_error"
	}

	return p.Var
}

func (p PermissionResult) IsOk() bool {
	return p.Var == "Ok" || p.Var == "OkWithMessage"
}
