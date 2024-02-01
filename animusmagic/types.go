package animusmagic

import (
	"errors"

	"github.com/anti-raid/splashtail/types/silverpelt"
)

var ErrNilRequestData = errors.New("request validation error: nil request data")
var ErrNilMessage = errors.New("request validation error: nil message")
var ErrNilClusterID = errors.New("request validation error: nil cluster id")
var ErrNilExpectedResponseCount = errors.New("request validation error: nil expected response count")
var ErrOpError = errors.New("request validation error: op is OpError")

const (
	OpRequest         = 0x0
	OpResponse        = 0x1
	OpError           = 0x2
	ScopeBot          = 0x0
	WildcardClusterID = 0xFFFF // top means wildcard/all clusters
)

type ClusterModules = []silverpelt.CanonicalModule

type AnimusResponse struct {
	Modules *struct {
		Modules ClusterModules `json:"modules"`
	}

	GuildsExist *struct {
		GuildsExist []uint8 `json:"guilds_exist"`
	}
}

type AnimusMessage struct {
	Modules     *struct{} `json:"Modules,omitempty"`
	GuildsExist *struct {
		Guilds []string `json:"guilds"`
	} `json:"GuildsExist,omitempty"`
}
