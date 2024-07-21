package silverpelt

import (
	"github.com/anti-raid/splashtail/core/go.std/ext_types"
	"github.com/bwmarrin/discordgo"
	orderedmap "github.com/wk8/go-ordered-map/v2"
)

type CanonicalSettingsResult struct {
	Ok *struct {
		Fields []orderedmap.OrderedMap[string, any] `json:"fields"`
	} `json:"Ok"`
	PermissionError *struct {
		Res PermissionResult `json:"res"`
	} `json:"PermissionError"`
	Err *struct {
		Error CanonicalSettingsError `json:"error"`
	} `json:"Err"`
}

type CanonicalSettingsError struct {
	OperationNotSupported *struct {
		Operation CanonicalOperationType `json:"operation"`
	} `json:"OperationNotSupported,omitempty"`
	Generic *struct {
		Message string `json:"message"`
		Src     string `json:"src"`
		Typ     string `json:"typ"`
	} `json:"Generic,omitempty"`
	SchemaTypeValidationError *struct {
		Column       string `json:"column"`
		ExpectedType string `json:"expected_type"`
		GotType      string `json:"got_type"`
	} `json:"SchemaTypeValidationError,omitempty"`
	SchemaNullValueValidationError *struct {
		Column string `json:"column"`
	} `json:"SchemaNullValueValidationError,omitempty"`
	SchemaCheckValidationError *struct {
		Column        string `json:"column"`
		Check         string `json:"check"`
		Error         string `json:"error"`
		AcceptedRange string `json:"accepted_range"`
	} `json:"SchemaCheckValidationError,omitempty"`
	MissingOrInvalidField *struct {
		Field string `json:"field"`
		Src   string `json:"src"`
	} `json:"MissingOrInvalidField,omitempty"`
	RowExists *struct {
		ColumnId string `json:"column_id"`
		Count    int64  `json:"count"`
	} `json:"RowExists,omitempty"`
	RowDoesNotExist *struct {
		ColumnId string `json:"column_id"`
	} `json:"RowDoesNotExist,omitempty"`
	MaximumCountReached *struct {
		Max     uint64 `json:"max"`
		Current uint64 `json:"current"`
	} `json:"MaximumCountReached,omitempty"`
}

type CanonicalColumnTypeDynamicClause struct {
	Field      string              `json:"field"`
	Value      any                 `json:"value"`
	ColumnType CanonicalColumnType `json:"column_type"`
}

type CanonicalColumnType struct {
	Scalar *struct {
		ColumnType CanonicalInnerColumnType `json:"column_type"`
	} `json:"Scalar,omitempty"`
	Array *struct {
		Inner CanonicalInnerColumnType `json:"inner"`
	} `json:"Array,omitempty"`
	Dynamic *struct {
		Clauses []CanonicalColumnTypeDynamicClause `json:"clauses"`
	}
}

type CanonicalInnerColumnTypeStringKindTemplateKind struct {
	// Template for formatting messages
	Message *struct{} `json:"Message,omitempty"`
}

type CanonicalInnerColumnTypeStringKind struct {
	Normal   *struct{} `json:"Normal,omitempty"`
	Textarea *struct{} `json:"Textarea,omitempty"`
	Template *struct {
		Kind CanonicalInnerColumnTypeStringKindTemplateKind `json:"kind"`
	} `json:"Template,omitempty"`
	User    *struct{} `json:"User,omitempty"`
	Channel *struct {
		AllowedTypes         []discordgo.ChannelType `json:"allowed_types"`
		NeededBotPermissions ext_types.Permissions   `json:"needed_bot_permissions"`
	} `json:"Channel,omitempty"`
	Role    *struct{} `json:"Role,omitempty"`
	Emoji   *struct{} `json:"Emoji,omitempty"`
	Message *struct{} `json:"Message,omitempty"`
}

type CanonicalInnerColumnType struct {
	Uuid   *struct{} `json:"Uuid,omitempty"`
	String *struct {
		MinLength     *int                               `json:"min_length,omitempty"`
		MaxLength     *int                               `json:"max_length,omitempty"`
		AllowedValues []string                           `json:"allowed_values,omitempty"`
		Kind          CanonicalInnerColumnTypeStringKind `json:"kind,omitempty"`
	} `json:"String,omitempty"`
	Timestamp   *struct{} `json:"Timestamp,omitempty"`
	TimestampTz *struct{} `json:"TimestampTz,omitempty"`
	Interval    *struct{} `json:"Interval,omitempty"`
	Integer     *struct{} `json:"Integer,omitempty"`
	Float       *struct{} `json:"Float,omitempty"`
	BitFlag     *struct {
		Values orderedmap.OrderedMap[string, int64] `json:"values"`
	} `json:"BitFlag,omitempty"`
	Boolean *struct{} `json:"Boolean,omitempty"`
	Json    *struct{} `json:"Json,omitempty"`
}

type CanonicalColumnSuggestion struct {
	Static *struct {
		Suggestions []string `json:"suggestions"`
	} `json:"Static,omitempty"`
	Dynamic *struct {
		// The table name to query
		TableName string `json:"table_name"`
		// The column name containing the ID/value to be set
		IDColumn string `json:"id_column"`
		// The column name containing the user-facing value
		ValueColumn string `json:"value_column"`
		// The column name containing the guild id
		GuildIDColumn string `json:"guild_id_column"`
	} `json:"Dynamic,omitempty"`
	None *struct{} `json:",omitempty"`
}

type CanonicalColumn struct {
	ID          string                    `json:"id"`
	Name        string                    `json:"name"`
	Description string                    `json:"description"`
	ColumnType  CanonicalColumnType       `json:"column_type"`
	Nullable    bool                      `json:"nullable"`
	Suggestions CanonicalColumnSuggestion `json:"suggestions"`
	Unique      bool                      `json:"unique"`
	IgnoredFor  []CanonicalOperationType  `json:"ignored_for"`
}

type CanonicalOperationSpecific struct {
	CorrespondingCommand string                                `json:"corresponding_command"`
	ColumnIDs            []string                              `json:"column_ids"`
	ColumnsToSet         orderedmap.OrderedMap[string, string] `json:"columns_to_set"`
}

type CanonicalOperationType string

const (
	View   CanonicalOperationType = "View"
	Create CanonicalOperationType = "Create"
	Update CanonicalOperationType = "Update"
	Delete CanonicalOperationType = "Delete"
)

func (c CanonicalOperationType) List() []string {
	return []string{
		"View",
		"Create",
		"Update",
		"Delete",
	}
}

func (c CanonicalOperationType) Parse() bool {
	for _, v := range c.List() {
		if v == string(c) {
			return true
		}
	}
	return false
}

type CanonicalConfigOption struct {
	ID            string                                                                    `json:"id"`
	Name          string                                                                    `json:"name"`
	Description   string                                                                    `json:"description"`
	Table         string                                                                    `json:"table"`
	GuildID       string                                                                    `json:"guild_id"`
	PrimaryKey    string                                                                    `json:"primary_key"`
	TitleTemplate string                                                                    `json:"title_template"`
	Columns       []CanonicalColumn                                                         `json:"columns"`
	MaxEntries    uint64                                                                    `json:"max_entries"`
	Operations    orderedmap.OrderedMap[CanonicalOperationType, CanonicalOperationSpecific] `json:"operations"`
}
