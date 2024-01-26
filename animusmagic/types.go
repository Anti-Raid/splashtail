package animusmagic

import (
	"errors"

	"github.com/anti-raid/splashtail/types/silverpelt"
	orderedmap "github.com/wk8/go-ordered-map/v2"
)

var ErrNilRequestData = errors.New("request validation error: nil request data")
var ErrNilMessage = errors.New("request validation error: nil message")
var ErrNilClusterID = errors.New("request validation error: nil cluster id")
var ErrNilExpectedResponseCount = errors.New("request validation error: nil expected response count")

const (
	OpRequest         = 0x0
	OpResponse        = 0x1
	WildcardClusterID = 0xFFFF // top means wildcard/all clusters
)

type ClusterModules = orderedmap.OrderedMap[string, silverpelt.CanonicalModule]

type AnimusResponse struct {
	Modules *struct {
		Modules ClusterModules `json:"modules"`
	}

	GuildsExist *struct {
		GuildsExist []uint8 `json:"guilds_exist"`
	}
}

type AnimusMessage struct {
	Modules     map[string]string
	GuildsExist *struct {
		GuildsExist []string `json:"guilds_exist"`
	}
}

// The Fix function is used to fix the AnimusMessage
// (Ready needs a non-nil map etc.)
func (c *AnimusMessage) Fix() {
	if c.Modules == nil {
		c.Modules = map[string]string{}
	}

	if c.GuildsExist == nil {
		c.GuildsExist = &struct {
			GuildsExist []string `json:"guilds_exist"`
		}{
			GuildsExist: []string{},
		}
	}
}
