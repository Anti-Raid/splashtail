package jobs

import (
	"errors"
	"fmt"
	"strings"

	"go.jobs/interfaces"
	"go.jobs/types"
	"go.std/splashcore"
	"go.std/utils"

	"golang.org/x/text/cases"
	"golang.org/x/text/language"
)

// Formats a Owner into a string under the 'normal' type. Returns nil if the Owner is nil or has an invalid target type
func FormatOwner(fu *types.Owner) (*string, error) {
	if fu == nil {
		return nil, errors.New("formatowner: jobs is nil")
	}

	switch fu.TargetType {
	case splashcore.TargetTypeUser:
		return utils.Pointer("u/" + fu.ID), nil
	case splashcore.TargetTypeServer:
		return utils.Pointer("g/" + fu.ID), nil
	default:
		return nil, fmt.Errorf("formatjobs: invalid target type: %s", fu.TargetType)
	}
}

// Parses a Owner from a string. Returns nil if the string is invalid.
//
// Owner must be in 'normal' (not simplex) form (e.g: u/1234567890).
func ParseOwner(fu string) *types.Owner {
	fuSplit := strings.SplitN(fu, "/", 2)

	if len(fuSplit) != 2 {
		return nil
	}

	switch fuSplit[0] {
	case "u":
		return &types.Owner{
			TargetType: splashcore.TargetTypeUser,
			ID:         fuSplit[1],
		}
	case "g":
		return &types.Owner{
			TargetType: splashcore.TargetTypeServer,
			ID:         fuSplit[1],
		}
	default:
		return nil
	}
}

// Formats in 'simplex' form (e.g: user/1234567890).
//
// This is mainly used for Object Storage and should NEVER be used for anything else especially database operations
func FormatOwnerSimplex(fu *types.Owner) string {
	if fu == nil {
		return ""
	}

	return cases.Lower(language.English).String(fu.TargetType) + "/" + fu.ID
}

func GetPathFromOutput(id string, jobImpl interfaces.JobImpl, outp *types.Output) string {
	if outp.Segregated {
		return fmt.Sprintf("%s/%s/%s/%s", FormatOwnerSimplex(jobImpl.Owner()), jobImpl.Name(), id, outp.Filename)
	} else {
		return fmt.Sprintf("jobs/%s", id)
	}
}
