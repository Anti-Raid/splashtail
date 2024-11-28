package silverpelt

import (
	orderedmap "github.com/wk8/go-ordered-map/v2"
)

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
	PermissionError *struct {
		Result PermissionResult `json:"result"`
	} `json:"PermissionError,omitempty"`
}

type CanonicalColumnType struct {
	Scalar *struct {
		Inner CanonicalInnerColumnType `json:"inner"`
	} `json:"Scalar,omitempty"`
	Array *struct {
		Inner CanonicalInnerColumnType `json:"inner"`
	} `json:"Array,omitempty"`
}

type CanonicalInnerColumnTypeStringKind struct {
	Normal *struct{} `json:"Normal,omitempty"`
	Token  *struct {
		DefaultLength uint64 `json:"default_length"`
	} `json:"Token,omitempty"`
	Textarea *struct {
		Ctx string `json:"ctx"`
	} `json:"Textarea,omitempty"`
	TemplateRef *struct {
		Kind string `json:"kind"`
		Ctx  string `json:"ctx"`
	} `json:"TemplateRef,omitempty"`
	User    *struct{} `json:"User,omitempty"`
	Role    *struct{} `json:"Role,omitempty"`
	Emoji   *struct{} `json:"Emoji,omitempty"`
	Message *struct{} `json:"Message,omitempty"`
	Channel *struct {
		NeededBotPermissions string   `json:"needed_bot_permissions"`
		AllowedChannelTypes  []string `json:"allowed_channel_types"`
	} `json:"Channel,omitempty"`
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
	Json    *struct {
		MaxBytes *int `json:"max_bytes"`
	} `json:"Json,omitempty"`
}

type CanonicalColumnSuggestion struct {
	Static *struct {
		Suggestions []string `json:"suggestions"`
	} `json:"Static,omitempty"`
	None *struct{} `json:",omitempty"`
}

type CanonicalColumn struct {
	ID          string                    `json:"id"`
	Name        string                    `json:"name"`
	Description string                    `json:"description"`
	ColumnType  CanonicalColumnType       `json:"column_type"`
	Nullable    bool                      `json:"nullable"`
	Suggestions CanonicalColumnSuggestion `json:"suggestions"`
	Secret      bool                      `json:"secret"`
	IgnoredFor  []CanonicalOperationType  `json:"ignored_for"`
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
	ID            string                   `json:"id"`
	Name          string                   `json:"name"`
	Description   string                   `json:"description"`
	PrimaryKey    string                   `json:"primary_key"`
	TitleTemplate string                   `json:"title_template"`
	Columns       []CanonicalColumn        `json:"columns"`
	Operations    []CanonicalOperationType `json:"operations"`
}
