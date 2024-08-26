package silverpelt

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestPermuteCommandNames(t *testing.T) {
	assert.Equal(t, PermuteCommandNames(""), []string{""})
	assert.Equal(t, PermuteCommandNames("limits"), []string{"limits"})
	assert.Equal(t, PermuteCommandNames("limits hit"), []string{"limits", "limits hit"})
	assert.Equal(t, PermuteCommandNames("limits hit add"), []string{"limits", "limits hit", "limits hit add"})
}
