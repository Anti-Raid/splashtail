package silverpelt

// CheckCommandOptions represents extra options for checking a command.
type CheckCommandOptions struct {
	// IgnoreModuleDisabled specifies whether or not to ignore the fact that the module is disabled in the guild.
	IgnoreModuleDisabled bool `json:"ignore_module_disabled,omitempty"`
	// IgnoreCommandDisabled specifies whether or not to ignore the fact that the command is disabled in the guild.
	IgnoreCommandDisabled bool `json:"ignore_command_disabled,omitempty"`
	// CustomResolvedKittyCatPerms specifies what custom resolved permissions to use for the user.
	// Note that EnsureUserHasCustomResolved must be true to ensure that the user has all the permissions in the custom_resolved_kittycat_perms.
	CustomResolvedKittyCatPerms []string `json:"custom_resolved_kittycat_perms,omitempty"`
	// EnsureUserHasCustomResolved specifies whether or not to ensure that the user has all the permissions in the custom_resolved_kittycat_perms.
	EnsureUserHasCustomResolved bool `json:"ensure_user_has_custom_resolved,omitempty"`
}
