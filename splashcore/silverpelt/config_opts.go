package silverpelt

import (
	orderedmap "github.com/wk8/go-ordered-map/v2"
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
		MinLength     *int     `json:"min_length,omitempty"`
		MaxLength     *int     `json:"max_length,omitempty"`
		AllowedValues []string `json:"allowed_values,omitempty"`
	} `json:"String,omitempty"`
	Timestamp   *struct{} `json:"Timestamp,omitempty"`
	TimestampTz *struct{} `json:"TimestampTz,omitempty"`
	Integer     *struct{} `json:"Integer,omitempty"`
	Float       *struct{} `json:"Float,omitempty"`
	BitFlag     *struct {
		Values orderedmap.OrderedMap[string, int64] `json:"values"`
	} `json:"BitFlag,omitempty"`
	Boolean *struct{} `json:"Boolean,omitempty"`
	User    *struct{} `json:"User,omitempty"`
	Channel *struct{} `json:"Channel,omitempty"`
	Role    *struct{} `json:"Role,omitempty"`
	Emoji   *struct{} `json:"Emoji,omitempty"`
	Message *struct{} `json:"Message,omitempty"`
	Json    *struct{} `json:"Json,omitempty"`
}

type CanonicalColumnSuggestion struct {
	Static *struct {
		Suggestions []string `json:"suggestions"`
	} `json:"Static,omitempty"`
	Dynamic *struct {
		TableName  string `json:"table_name"`
		ColumnName string `json:"column_name"`
	} `json:"Dynamic,omitempty"`
	None *struct{} `json:",omitempty"`
}

type CanonicalColumn struct {
	ID          string                                              `json:"id"`
	Name        string                                              `json:"name"`
	ColumnType  CanonicalColumnType                                 `json:"column_type"`
	Nullable    bool                                                `json:"nullable"`
	Suggestions CanonicalColumnSuggestion                           `json:"suggestions"`
	Unique      bool                                                `json:"unique"`
	Readonly    orderedmap.OrderedMap[CanonicalOperationType, bool] `json:"readonly"`
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
