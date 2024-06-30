package bigint

import (
	"fmt"
	"math/big"
)

type BigInt struct {
	big.Int
}

func (b BigInt) MarshalJSON() ([]byte, error) {
	return []byte("\"" + b.String() + "\""), nil
}

func (b *BigInt) UnmarshalJSON(p []byte) error {
	if len(p) == 0 || string(p) == "null" {
		return nil
	}

	if p[0] == '"' {
		if len(p) == 1 {
			return fmt.Errorf("invalid big integer [len(p) == 1]: %s", p)
		}

		// Ensure last char is a "
		if p[len(p)-1] != '"' {
			return fmt.Errorf("invalid big integer: %s", p)
		}

		p = p[1 : len(p)-1]
	}

	var z big.Int
	_, ok := z.SetString(string(p), 10)
	if !ok {
		return fmt.Errorf("not a valid big integer: %s", p)
	}
	b.Int = z
	return nil
}
