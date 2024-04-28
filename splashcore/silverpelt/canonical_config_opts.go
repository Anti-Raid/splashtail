package silverpelt

type CanonicalColumnType string

const (
	String  CanonicalColumnType = "String"
	Integer CanonicalColumnType = "Integer"
	Boolean CanonicalColumnType = "Boolean"
	User    CanonicalColumnType = "User"
	Channel CanonicalColumnType = "Channel"
	Role    CanonicalColumnType = "Role"
	Emoji   CanonicalColumnType = "Emoji"
	Message CanonicalColumnType = "Message"
)

type CanonicalColumn struct {
	ID         string              `json:"id"`          // The ID of the column
	Name       string              `json:"name"`        // The friendly name of the column
	ColumnType CanonicalColumnType `json:"column_type"` // The type of the column
	Nullable   bool                `json:"nullable"`    // Whether or not the column is nullable
	Unique     bool                `json:"unique"`      // Whether or not the column is unique
	Array      bool                `json:"array"`       // Whether or not the column is an array
	Hint       *string             `json:"hint"`        // Column hint, used internally for stuff like autocomplete
}

type CanonicalConfigOption struct {
	ID           string            `json:"id"`             // The ID of the option
	Name         string            `json:"name"`           // The name of the option
	Description  string            `json:"description"`    // The description of the option
	Table        string            `json:"table"`          // The table name for the config option
	GuildId      string            `json:"guild_id"`       // The column name refering to the guild id of the config option
	Columns      []CanonicalColumn `json:"columns"`        // The columns for the option
	RowMustExist bool              `json:"row_must_exist"` // Whether or not the row must exist before hand
	Hint         *string           `json:"hint"`           // Config option hint, used internally for stuff like guild channel configuration
}
