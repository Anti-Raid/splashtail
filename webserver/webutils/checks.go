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
		pc.ChecksNeeded = 1
	}

	if len(pc.Checks) > maxPermCheck {
		return nil, fmt.Errorf("too many checks: %d", len(pc.Checks))
	}

	var parsedChecks = make([]silverpelt.PermissionCheck, len(pc.Checks))
	for i, check := range pc.Checks {
		if len(check.KittycatPerms) == 0 && len(check.NativePerms) == 0 {
			continue
		}

		parsedChecks[i] = silverpelt.PermissionCheck{
			KittycatPerms: check.KittycatPerms,
			NativePerms:   check.NativePerms,
			OuterAnd:      check.OuterAnd,
			InnerAnd:      check.InnerAnd,
		}

		if len(parsedChecks[i].KittycatPerms) > maxKittycatPerms {
			return nil, fmt.Errorf("too many kittycat perms: %d", len(parsedChecks[i].KittycatPerms))
		}

		if len(parsedChecks[i].NativePerms) > maxNativePerms {
			return nil, fmt.Errorf("too many native perms: %d", len(parsedChecks[i].NativePerms))
		}

		for j := range parsedChecks[i].NativePerms {
			parsedChecks[i].NativePerms[j] = ParseBitFlag(state.SerenityPermissions, parsedChecks[i].NativePerms[j])
		}

		for _, perm := range parsedChecks[i].KittycatPerms {
			if len(perm) > maxIndividualKittycatPermSize {
				return nil, fmt.Errorf("kittycat perm too long: max=%d", maxIndividualKittycatPermSize)
			}
		}
	}

	return &silverpelt.PermissionChecks{
		Checks:       parsedChecks,
		ChecksNeeded: pc.ChecksNeeded,
	}, nil
}
