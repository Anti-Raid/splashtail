package animusmagic

import (
	"errors"
	"strings"
)

var ErrNilRequestData = errors.New("request validation error: nil request data")
var ErrNilMessage = errors.New("request validation error: nil message")
var ErrInvalidPayload = errors.New("request validation error: invalid payload")
var ErrNilClusterID = errors.New("request validation error: nil cluster id")
var ErrNilExpectedResponseCount = errors.New("request validation error: nil expected response count")
var ErrInvalidTarget = errors.New("request validation error: message target and payload.To mismatch")
var ErrOpError = errors.New("request validation error: op is OpError")

type AnimusTarget byte

const (
	AnimusTargetBot       AnimusTarget = 0x0
	AnimusTargetJobserver AnimusTarget = 0x1
	AnimusTargetWebserver AnimusTarget = 0x2
	AnimusTargetWildcard  AnimusTarget = 0xFF
)

func (a AnimusTarget) String() string {
	switch a {
	case AnimusTargetBot:
		return "Bot"
	case AnimusTargetJobserver:
		return "Jobserver"
	case AnimusTargetWebserver:
		return "Webserver"
	case AnimusTargetWildcard:
		return "Wildcard"
	default:
		return "Unknown"
	}
}

func ByteToAnimusTarget(b uint8) (AnimusTarget, bool) {
	switch b {
	case 0x0:
		return AnimusTargetBot, true
	case 0x1:
		return AnimusTargetJobserver, true
	case 0x2:
		return AnimusTargetWebserver, true
	default:
		return 0, false
	}
}

func StringToAnimusTarget(s string) (AnimusTarget, bool) {
	var strMap = map[string]AnimusTarget{
		"0":         AnimusTargetBot,
		"bot":       AnimusTargetBot,
		"1":         AnimusTargetJobserver,
		"jobserver": AnimusTargetJobserver,
		"jobs":      AnimusTargetJobserver,
		"2":         AnimusTargetWebserver,
		"webserver": AnimusTargetWebserver,
		"web":       AnimusTargetWebserver,
		"api":       AnimusTargetWebserver,
	}

	if val, ok := strMap[strings.ToLower(s)]; ok {
		return val, true
	}

	return 0, false
}

type AnimusOp byte

const (
	OpRequest  AnimusOp = 0x0
	OpResponse AnimusOp = 0x1
	OpError    AnimusOp = 0x2
)

func (a AnimusOp) String() string {
	switch a {
	case OpRequest:
		return "Request"
	case OpResponse:
		return "Response"
	case OpError:
		return "Error"
	default:
		return "Unknown"
	}
}

func ByteToAnimusOp(b uint8) (AnimusOp, bool) {
	switch b {
	case 0x0:
		return OpRequest, true
	case 0x1:
		return OpResponse, true
	case 0x2:
		return OpError, true
	default:
		return 0, false
	}
}

func StringToAnimusOp(s string) (AnimusOp, bool) {
	var strMap = map[string]AnimusOp{
		"0":        OpRequest,
		"request":  OpRequest,
		"req":      OpRequest,
		"1":        OpResponse,
		"response": OpResponse,
		"resp":     OpResponse,
		"2":        OpError,
		"error":    OpError,
		"err":      OpError,
	}

	if val, ok := strMap[strings.ToLower(s)]; ok {
		return val, true
	}

	return 0, false
}

const (
	WildcardClusterID = 0xFFFF // top means wildcard/all clusters
)

type AnimusMessageMetadata struct {
	From          AnimusTarget
	To            AnimusTarget
	ClusterID     uint16
	Op            AnimusOp
	CommandID     string
	PayloadOffset uint
}

type AnimusErrorResponse struct {
	Message string `json:"message"`
	Context string `json:"context"`

	// Client internal
	ClientDebugInfo map[string]any `json:"client_debug_info,omitempty"`
}

type AnimusMessage interface {
	Message()             // Marker method
	Target() AnimusTarget // Who the message is for
}

type AnimusResponse interface {
	Response()            // Marker method
	Target() AnimusTarget // Who can create a response should be from
}
