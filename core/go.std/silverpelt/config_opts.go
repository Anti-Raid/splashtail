package silverpelt

import (
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
	} `json:"OperationNotSupported"`
	Generic *struct {
		Message string `json:"message"`
		Src     string `json:"src"`
		Typ     string `json:"typ"`
	} `json:"Generic"`
	SchemaTypeValidationError *struct {
		Column       string `json:"column"`
		ExpectedType string `json:"expected_type"`
		GotType      string `json:"got_type"`
	} `json:"SchemaTypeValidationError"`
	SchemaNullValueValidationError *struct {
		Column string `json:"column"`
	} `json:"SchemaNullValueValidationError"`
	SchemaCheckValidationError *struct {
		Column        string `json:"column"`
		Check         string `json:"check"`
		Error         string `json:"error"`
		AcceptedRange string `json:"accepted_range"`
	} `json:"SchemaCheckValidationError"`
	MissingOrInvalidField *struct {
		Field string `json:"field"`
		Src   string `json:"src"`
	} `json:"MissingOrInvalidField"`
	RowExists *struct {
		ColumnId string `json:"column_id"`
		Count    int64  `json:"count"`
	} `json:"RowExists"`
	RowDoesNotExist *struct {
		ColumnId string `json:"column_id"`
	} `json:"RowDoesNotExist"`
	MaximumCountReached *struct {
		Max     uint64 `json:"max"`
		Current uint64 `json:"current"`
	} `json:"MaximumCountReached"`
}

type CanonicalInnerColumnTypeStringKind string

const (
	CanonicalInnerColumnTypeStringKindNormal  CanonicalInnerColumnTypeStringKind = "Normal"
	CanonicalInnerColumnTypeStringKindUser    CanonicalInnerColumnTypeStringKind = "User"
	CanonicalInnerColumnTypeStringKindChannel CanonicalInnerColumnTypeStringKind = "Channel"
	CanonicalInnerColumnTypeStringKindRole    CanonicalInnerColumnTypeStringKind = "Role"
	CanonicalInnerColumnTypeStringKindEmoji   CanonicalInnerColumnTypeStringKind = "Emoji"
	CanonicalInnerColumnTypeStringKindMessage CanonicalInnerColumnTypeStringKind = "Message"
)

type CanonicalColumnType struct {
	Scalar *struct {
		ColumnType CanonicalInnerColumnType `json:"column_type"`
	} `json:"Scalar,omitempty"`
	Array *struct {
		Inner CanonicalInnerColumnType `json:"inner"`
	} `json:"Array,omitempty"`
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
	ID          string                                                                    `json:"id"`
	Name        string                                                                    `json:"name"`
	Description string                                                                    `json:"description"`
	Table       string                                                                    `json:"table"`
	GuildID     string                                                                    `json:"guild_id"`
	PrimaryKey  string                                                                    `json:"primary_key"`
	Columns     []CanonicalColumn                                                         `json:"columns"`
	Operations  orderedmap.OrderedMap[CanonicalOperationType, CanonicalOperationSpecific] `json:"operations"`
}
