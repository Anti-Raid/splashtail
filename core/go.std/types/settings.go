package types

import (
	"github.com/anti-raid/splashtail/core/go.std/silverpelt"
	orderedmap "github.com/wk8/go-ordered-map/v2"
)

// SettingsExecute allows execution of a settings operation
type SettingsExecute struct {
	Operation silverpelt.CanonicalOperationType  `json:"operation" description:"The operation type to execute"`
	Module    string                             `json:"module" description:"The module in which the setting is in"`
	Setting   string                             `json:"setting" description:"The name of the setting"`
	Fields    orderedmap.OrderedMap[string, any] `json:"fields" description:"The fields to execute the operation with"`
}

type SettingsExecuteResponse struct {
	Fields []orderedmap.OrderedMap[string, any] `json:"fields" description:"The fields returned by the operation"`
}
