// Animus Magic is the internal redis IPC system for internal communications between the bot and the server
//
// Format of payloads: <scope: u8><cluster id: u16><op: 8 bits><command id: alphanumeric string>/<cbor payload>
package animusmagic

import (
	"context"
	"time"

	"github.com/anti-raid/splashtail/utils/syncmap"
	"github.com/fxamacker/cbor/v2"
	"github.com/infinitybotlist/eureka/crypto"
	"github.com/redis/rueidis"
	"go.uber.org/zap"
)

const ChannelName = "animus_magic"

type ClientCache struct {
	ClusterModules syncmap.Map[uint16, *ClusterModules]
}

type ClientResponse struct {
	ClusterID uint16
	Scope     byte
	Op        byte
	Error     map[string]string // Only applicable if Op is OpError
	Resp      *AnimusResponse
}

type AnimusMagicClient struct {
	// All data that is stored on the cache
	Cache *ClientCache

	// Set of notifiers
	Notify syncmap.Map[string, chan *ClientResponse]
}

// New returns a new AnimusMagicClient
func New() *AnimusMagicClient {
	return &AnimusMagicClient{
		Cache: &ClientCache{
			ClusterModules: syncmap.Map[uint16, *ClusterModules]{},
		},
	}
}

// UpdateCache updates the cache with the given response
func (c *AnimusMagicClient) UpdateCache(resp *ClientResponse) {
	if resp.Resp.Modules != nil {
		c.Cache.ClusterModules.Store(resp.ClusterID, &resp.Resp.Modules.Modules)
	}
}

// ListenOnce starts listening for messages from redis
//
// This is *blocking* and should be run in a goroutine
func (c *AnimusMagicClient) ListenOnce(ctx context.Context, redis rueidis.Client, l *zap.Logger) error {
	return redis.Dedicated(
		func(redis rueidis.DedicatedClient) error {
			return redis.Receive(ctx, redis.B().Subscribe().Channel(ChannelName).Build(), func(msg rueidis.PubSubMessage) {
				bytesData := []byte(msg.Message)

				// Below 5 is impossible
				if len(bytesData) < 5 {
					return
				}

				// First byte is the scope
				scope := bytesData[0]

				// Next 2 bytes are the cluster id
				clusterId := uint16(bytesData[1])<<8 | uint16(bytesData[2])

				// Next byte is the op
				op := bytesData[3]

				if op != OpResponse && op != OpError {
					return
				}

				go func() {
					// Next bytes are the command id but only till '/'
					commandId := ""
					for i := 4; i < len(bytesData); i++ {
						if bytesData[i] == '/' {
							break
						}
						commandId += string(bytesData[i])
					}

					// Rest is the payload
					payload := bytesData[len(commandId)+5:]

					var response *ClientResponse

					if op == OpResponse {
						var resp *AnimusResponse
						err := cbor.Unmarshal(payload, &resp)

						if err != nil {
							l.Error("[animus magic] error unmarshaling payload", zap.Error(err))
							return
						}

						response = &ClientResponse{
							Scope:     scope,
							ClusterID: clusterId,
							Op:        op,
							Resp:      resp,
						}
					} else {
						var data map[string]string
						err := cbor.Unmarshal(payload, &data)

						if err != nil {
							data = map[string]string{
								"client_error": "failed to unmarshal error payload:" + err.Error(),
							}
						}

						response = &ClientResponse{
							Scope:     scope,
							ClusterID: clusterId,
							Op:        op,
							Error:     data,
						}
					}

					n, ok := c.Notify.Load(commandId)

					if !ok {
						l.Warn("[animus magic] received response for unknown command", zap.String("commandId", commandId))
						if response.Op == OpResponse {
							c.UpdateCache(response) // At least update the cache
						}
						return
					}

					n <- response
					c.Notify.Delete(commandId)
					if response.Op == OpResponse {
						c.UpdateCache(response)
					}
				}()
			})
		},
	)
}

// Listen starts listening for messages from redis
// and restarts the listener if it dies
func (c *AnimusMagicClient) Listen(ctx context.Context, redis rueidis.Client, l *zap.Logger) error {
	for {
		err := c.ListenOnce(ctx, redis, l)
		if err != nil {
			l.Error("[animus magic] error listening to redis", zap.Error(err))
			time.Sleep(1 * time.Second)
		}
	}
}

// CreatePayload creates a payload for the given command id and message
func (c *AnimusMagicClient) CreatePayload(scope byte, clusterId uint16, op byte, commandId string, resp *AnimusMessage) ([]byte, error) {
	var finalPayload = []byte{
		scope,
		byte(clusterId>>8) & 0xFF,
		byte(clusterId) & 0xFF,
		op,
	}

	finalPayload = append(finalPayload, []byte(commandId+"/")...)

	payload, err := cbor.Marshal(resp)

	if err != nil {
		return nil, err
	}

	finalPayload = append(finalPayload, payload...)

	return finalPayload, nil
}

func (c *AnimusMagicClient) publish(ctx context.Context, redis rueidis.Client, payload []byte) error {
	return redis.Do(ctx, redis.B().Publish().Channel(ChannelName).Message(rueidis.BinaryString(payload)).Build()).Error()
}

// RequestData stores the data for a request
type RequestData struct {
	ClusterID             *uint16        // must be set, also ExpectedResponseCount must be set if wildcard
	ExpectedResponseCount *int           // must be set if wildcard. this is the number of responses expected
	CommandID             string         // if unset, will be randomly generated
	Scope                 *byte          // if unset, will be set to ScopeBot
	Op                    *byte          // if unset, will be set to OpRequest
	IgnoreOpError         bool           // if true, will ignore OpError responses
	Message               *AnimusMessage // must be set
}

// Request sends a request to the given cluster id and waits for a response
func (c *AnimusMagicClient) Request(ctx context.Context, redis rueidis.Client, data *RequestData) ([]*ClientResponse, error) {
	if data == nil {
		return nil, ErrNilRequestData
	}

	if data.ClusterID == nil {
		return nil, ErrNilClusterID
	}

	if *data.ClusterID == WildcardClusterID && data.ExpectedResponseCount == nil {
		return nil, ErrNilExpectedResponseCount
	}

	if data.Message == nil {
		return nil, ErrNilMessage
	}

	if data.CommandID == "" {
		data.CommandID = crypto.RandString(16)
	}

	if data.Scope == nil {
		data.Scope = new(byte) // 0 is the default value of byte which corresponds to ScopeBot
	}

	if data.Op == nil {
		data.Op = new(byte) // 0 is the default value of byte which corresponds to OpRequest
	}

	payload, err := c.CreatePayload(*data.Scope, *data.ClusterID, *data.Op, data.CommandID, data.Message)

	if err != nil {
		return nil, err
	}

	// Create a channel to receive the response
	size := 1 // default buffer size is 1

	if *data.ClusterID == WildcardClusterID {
		size = 0 // if wildcard, then we don't know how many responses we'll get
	}

	notify := make(chan *ClientResponse, size)

	// Store the channel in the notify map
	c.Notify.Store(data.CommandID, notify)

	// Publish the payload
	err = c.publish(ctx, redis, payload)

	if err != nil {
		return nil, err
	}

	// Wait for the response
	var resps []*ClientResponse

	for {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case resp := <-notify:
			resps = append(resps, resp)

			if resp.Op == OpError && !data.IgnoreOpError {
				return resps, ErrOpError
			}

			if data.ExpectedResponseCount != nil && *data.ExpectedResponseCount > 1 {
				if len(resps) >= *data.ExpectedResponseCount {
					return resps, nil
				}
			} else {
				return resps, nil
			}
		}
	}
}
