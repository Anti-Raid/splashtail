package silverpelt

import (
	"fmt"
	"math/big"

	"go.std/bigint"
)

// Returns the extended data for a command
func GetCommandExtendedData(
	permutations []string,
	commandExtendedDataMap CommandExtendedDataMap,
) *CommandExtendedData {
	rootCmd := permutations[0]

	var cmdData *CommandExtendedData

	cmdDataVal, ok := commandExtendedDataMap.Get("")

	if !ok {
		cmdData = &CommandExtendedData{
			DefaultPerms: PermissionCheck{
				KittycatPerms: []string{fmt.Sprintf("%s.%s", rootCmd, "*")},
				NativePerms: []bigint.BigInt{
					{
						Int: *big.NewInt(8),
					},
				},
			},
			IsDefaultEnabled: true,
			WebHidden:        false,
			VirtualCommand:   false,
		}
	} else {
		cmdData = &cmdDataVal
	}

	for _, command := range permutations {
		cmdReplaced := command[len(rootCmd):]

		if data, ok := commandExtendedDataMap.Get(cmdReplaced); ok {
			cmdData = &data
		}
	}

	return cmdData
}
