package types

import (
	"github.com/Anti-Raid/corelib_go/silverpelt"
	orderedmap "github.com/wk8/go-ordered-map/v2"
)

// SettingsExecute allows execution of a settings operation
type SettingsExecute struct {
	Operation silverpelt.CanonicalOperationType  `json:"operation" description:"The operation type to execute"`
	Setting   string                             `json:"setting" description:"The name of the setting"`
	Fields    orderedmap.OrderedMap[string, any] `json:"fields" description:"The fields to execute the operation with"`
}

// SettingsExecuteResponse is the response to a settings operation
type SettingsExecuteResponse struct {
	Fields []orderedmap.OrderedMap[string, any] `json:"fields" description:"The fields returned by the operation"`
}

// SettingsGetSuggestions allows getting dynamic suggestions for a setting
type SettingsGetSuggestions struct {
	Operation silverpelt.CanonicalOperationType `json:"operation" description:"The operation type to execute"`
	Module    string                            `json:"module" description:"The module in which the setting is in"`
	Setting   string                            `json:"setting" description:"The ID of the setting"`
	Column    string                            `json:"column" description:"The column to get suggestions for"`
	Filter    *string                           `json:"filter,omitempty" description:"The filter to apply to the suggestions. If null, no filter is applied"`
}

// SettingsGetSuggestionSuggestion is a suggestion for a setting
type SettingsGetSuggestionSuggestion struct {
	ID    any `json:"id" description:"The ID of the suggestion"`
	Value any `json:"value" description:"The value of the suggestion"`
}

type SettingsGetSuggestionsResponse struct {
	Suggestions []SettingsGetSuggestionSuggestion `json:"suggestions" description:"The suggestions for the setting"`
}
