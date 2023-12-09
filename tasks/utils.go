package tasks

import (
	"fmt"
	"splashtail/types"
	"strings"

	"golang.org/x/text/cases"
	"golang.org/x/text/language"
)

func FormatTaskFor(fu *types.TaskFor) *string {
	if fu == nil {
		return nil
	}

	switch fu.TargetType {
	case types.TargetTypeUser:
		return Pointer("u/" + fu.ID)
	case types.TargetTypeServer:
		return Pointer("g/" + fu.ID)
	default:
		return nil
	}
}

func ParseTaskFor(fu string) *types.TaskFor {
	fuSplit := strings.SplitN(fu, "/", 2)

	if len(fuSplit) != 2 {
		return nil
	}

	switch fuSplit[0] {
	case "u":
		return &types.TaskFor{
			TargetType: types.TargetTypeUser,
			ID:         fuSplit[1],
		}
	case "g":
		return &types.TaskFor{
			TargetType: types.TargetTypeServer,
			ID:         fuSplit[1],
		}
	default:
		return nil
	}
}

func FormatTaskForSimplex(fu *types.TaskFor) string {
	if fu == nil {
		return ""
	}

	return cases.Lower(language.English).String(fu.TargetType) + "/" + fu.ID
}

func GetPathFromOutput(taskId string, tInfo *types.TaskInfo, outp *types.TaskOutput) string {
	if outp.Segregated {
		return fmt.Sprintf("%s/%s/%s/%s", FormatTaskForSimplex(tInfo.TaskFor), tInfo.Name, taskId, outp.Filename)
	} else {
		return fmt.Sprintf("tasks/%s", taskId)
	}
}
