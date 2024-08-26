package apps

import (
	"go.api/state"
	"go.api/types"
)

var Stable = true

func Setup() {
	for _, app := range Apps {
		appValidator := state.Validator.Struct(app)

		if appValidator != nil {
			panic("App validation failed: " + appValidator.Error())
		}
	}
}

func FindPosition(id string) *types.Position {
	for _, pos := range Apps {
		if pos.ID == id {
			return &pos
		}
	}

	return nil
}
