package silverpelt

import (
	"strings"
)

// From name_split, construct a list of all permutations of the command name from the root till the end
//
// E.g: If subcommand is `limits hit`, then `limits` and `limits hit` will be constructed
//
//	as the list of commands to check
//
// E.g 2: If subcommand is `limits hit add`, then `limits`, `limits hit` and `limits hit add`
//
//	will be constructed as the list of commands to check
func PermuteCommandNames(name string) []string {
	// Check if subcommand by splitting the command name
	nameSplit := strings.Split(name, " ")

	var commandsToCheck []string

	for i := 0; i < len(nameSplit); i++ {
		var command string

		for j, cmd := range nameSplit[:i+1] {
			command += cmd

			if j != i {
				command += " "
			}
		}

		commandsToCheck = append(commandsToCheck, command)
	}

	return commandsToCheck
}
