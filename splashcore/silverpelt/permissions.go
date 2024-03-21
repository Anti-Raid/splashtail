// From botv2 silverpelt/permissions.rs
package silverpelt

import "fmt"

type PermissionResult struct {
	Var           string                     `json:"var"`
	Message       string                     `json:"message"`
	Check         *PermissionCheck           `json:"check"`
	CommandConfig *GuildCommandConfiguration `json:"command_config"`
	ModuleConfig  *GuildModuleConfiguration  `json:"module_config"`
	Checks        *PermissionChecks          `json:"checks"`
	Error         string                     `json:"error"`
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
	case "ModuleDisabled":
		return "module_disabled"
	case "NoChecksSucceeded":
		return "no_checks_succeeded"
	case "MissingMinChecks":
		return "missing_min_checks"
	case "DiscordError":
		return "discord_error"
	case "GenericError":
		return "generic_error"
	}

	return p.Var
}

func (p PermissionResult) IsOk() bool {
	return p.Var == "Ok" || p.Var == "OkWithMessage"
}
