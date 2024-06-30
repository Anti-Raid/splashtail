// From silverpelt/canonical_module
package silverpelt

import orderedmap "github.com/wk8/go-ordered-map/v2"

type CommandExtendedDataMap = orderedmap.OrderedMap[string, CommandExtendedData]

type CanonicalModule struct {
	ID                 string                  `json:"id"`
	Name               string                  `json:"name"`
	Description        string                  `json:"description"`
	Toggleable         bool                    `json:"toggleable"`
	CommandsToggleable bool                    `json:"commands_toggleable"`
	WebHidden          bool                    `json:"web_hidden"`
	VirtualModule      bool                    `json:"virtual_module"`
	IsDefaultEnabled   bool                    `json:"is_default_enabled"`
	Commands           []CanonicalCommand      `json:"commands"`
	S3Paths            []string                `json:"s3_paths"`
	ConfigOptions      []CanonicalConfigOption `json:"config_options"`
}

type CanonicalCommand struct {
	Command      CanonicalCommandData   `json:"command"`
	ExtendedData CommandExtendedDataMap `json:"extended_data"`
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
