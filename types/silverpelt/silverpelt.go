package silverpelt

type PermissionCheck struct {
	KittycatPerms []string `json:"kittycat_perms"`
	NativePerms   []string `json:"native_perms"`
	OuterAnd      bool     `json:"outer_and"`
	InnerAnd      bool     `json:"inner_and"`
}

type PermissionChecks struct {
	Checks       []PermissionCheck `json:"checks"`
	ChecksNeeded int               `json:"checks_needed"`
}

type CanonicalCommandExtendedData struct {
	ID           string           `json:"id"`
	DefaultPerms PermissionChecks `json:"default_perms"`
}

type CanonicalModule struct {
	ID                   string             `json:"id"`
	Name                 string             `json:"name"`
	Description          string             `json:"description"`
	Configurable         bool               `json:"configurable"`
	CommandsConfigurable bool               `json:"commands_configurable"`
	WebHidden            bool               `json:"web_hidden"`
	IsDefaultEnabled     bool               `json:"is_default_enabled"`
	Commands             []CanonicalCommand `json:"commands"`
}

type CanonicalCommand struct {
	Command      CanonicalCommandData           `json:"command"`
	ExtendedData []CanonicalCommandExtendedData `json:"extended_data"`
}

type CanonicalCommandArgument struct {
	Name        string   `json:"name"`
	Description *string  `json:"description"`
	Required    bool     `json:"required"`
	Choices     []string `json:"choices"`
}

type CanonicalCommandData struct {
	Name               string                     `json:"name"`
	QualifiedName      string                     `json:"qualified_name"`
	Description        *string                    `json:"description"`
	NSFW               bool                       `json:"nsfw"`
	Subcommands        []CanonicalCommandData     `json:"subcommands"`
	SubcommandRequired bool                       `json:"subcommand_required"`
	Arguments          []CanonicalCommandArgument `json:"arguments"`
}
