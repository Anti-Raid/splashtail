package animusmagic

import (
	"errors"
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
)

const (
	OpRequest         = 0x0
	OpResponse        = 0x1
	OpError           = 0x2
	WildcardClusterID = 0xFFFF // top means wildcard/all clusters
)

type AnimusMessageMetadata struct {
	From          AnimusTarget
	To            AnimusTarget
	ClusterID     uint16
	Op            byte
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
