package webutils

import (
	"fmt"
	"math/big"

	"github.com/anti-raid/splashtail/splashcore/bigint"
	"github.com/anti-raid/splashtail/splashcore/silverpelt"
	"github.com/anti-raid/splashtail/webserver/state"
)

const (
	maxPermCheck                  = 10
	maxKittycatPerms              = 10
	maxIndividualKittycatPermSize = 128
	maxNativePerms                = 10
)

// ParseBitFlag returns a new bitfield with the flags that are set in the given flags map.
func ParseBitFlag(flags map[string]bigint.BigInt, flag bigint.BigInt) bigint.BigInt {
	var parsedFlag = bigint.BigInt{}

	for _, v := range flags {
		tempFlag := big.Int{}
		if tempFlag.And(&v.Int, &flag.Int).Cmp(&v.Int) == 0 {
			parsedFlag.Or(&parsedFlag.Int, &v.Int)
		}
	}

	return parsedFlag
}

// Parses a user-inputted PermissionChecks object into a parsed PermissionChecks object.
func ParsePermissionChecks(pc *silverpelt.PermissionChecks) (*silverpelt.PermissionChecks, error) {
	if pc == nil {
		return nil, fmt.Errorf("pc is nil")
	}

	if pc.ChecksNeeded < 1 {
		return nil, fmt.Errorf("checks_needed must be at least 1")
	}

	if len(pc.Checks) > maxPermCheck {
		return nil, fmt.Errorf("too many checks: %d", len(pc.Checks))
	}

	var parsedChecks = make([]silverpelt.PermissionCheck, 0, len(pc.Checks))
	for _, check := range pc.Checks {
		if len(check.KittycatPerms) == 0 && len(check.NativePerms) == 0 {
			continue
		}

		parsedCheck := silverpelt.PermissionCheck{
			KittycatPerms: func() []string {
				if len(check.KittycatPerms) == 0 {
					return make([]string, 0)
				}

				return check.KittycatPerms
			}(),
			NativePerms: func() []bigint.BigInt {
				if len(check.NativePerms) == 0 {
					return make([]bigint.BigInt, 0)
				}

				return check.NativePerms
			}(),
			OuterAnd: check.OuterAnd,
			InnerAnd: check.InnerAnd,
		}

		if len(parsedCheck.KittycatPerms) > maxKittycatPerms {
			return nil, fmt.Errorf("too many kittycat perms: %d", len(parsedCheck.KittycatPerms))
		}

		if len(parsedCheck.NativePerms) > maxNativePerms {
			return nil, fmt.Errorf("too many native perms: %d", len(parsedCheck.NativePerms))
		}

		for j := range parsedCheck.NativePerms {
			parsedCheck.NativePerms[j] = ParseBitFlag(state.SerenityPermissions, parsedCheck.NativePerms[j])
		}

		for _, perm := range parsedCheck.KittycatPerms {
			if len(perm) > maxIndividualKittycatPermSize {
				return nil, fmt.Errorf("kittycat perm too long: max=%d", maxIndividualKittycatPermSize)
			}
		}

		parsedChecks = append(parsedChecks, parsedCheck)
	}

	return &silverpelt.PermissionChecks{
		Checks:       parsedChecks,
		ChecksNeeded: pc.ChecksNeeded,
	}, nil
}
