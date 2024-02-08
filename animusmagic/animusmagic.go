// Animus Magic is the internal redis IPC system for internal communications between the bot and the server
//
// Format of payloads: <scope: u8><cluster id: u16><op: 8 bits><command id: alphanumeric string>/<cbor payload>
package animusmagic

import (
	"context"
	"sync/atomic"
	"time"

	"github.com/anti-raid/splashtail/utils/syncmap"
	"github.com/fxamacker/cbor/v2"
	"github.com/infinitybotlist/eureka/crypto"
	"github.com/redis/rueidis"
	"go.uber.org/zap"
)

type ClientCache struct {
	ClusterModules syncmap.Map[uint16, *ClusterModules]
}

type ClientResponse struct {
	ClusterID uint16
	Scope     byte
	Op        byte
	Error     *AnimusErrorResponse // Only applicable if Op is OpError
	Resp      *AnimusResponse
}

type NotifyWrapper struct {
	Chan          chan *ClientResponse
	ExpectedCount uint32
	ResponseCount atomic.Uint32
}

type AnimusMagicClient struct {
	// All data that is stored on the cache
	Cache *ClientCache

	// Set of notifiers
	Notify syncmap.Map[string, *NotifyWrapper]

	// The redis channel to use
	Channel string
}

// New returns a new AnimusMagicClient
func New(channel string) *AnimusMagicClient {
	return &AnimusMagicClient{
		Cache: &ClientCache{
			ClusterModules: syncmap.Map[uint16, *ClusterModules]{},
		},
		Channel: channel,
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
			return redis.Receive(ctx, redis.B().Subscribe().Channel(c.Channel).Build(), func(msg rueidis.PubSubMessage) {
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

							// Try getting better debug data
							var cdr map[string]any

							err2 := cbor.Unmarshal(payload, &cdr)

							response = &ClientResponse{
								Scope:     scope,
								ClusterID: clusterId,
								Op:        OpError,
								Error: &AnimusErrorResponse{
									Message: "client error: error unmarshaling payload: " + err.Error(),
									ClientDebugInfo: map[string]any{
										"data": cdr,
										"err":  err2,
									},
								},
							}
						} else {
							response = &ClientResponse{
								Scope:     scope,
								ClusterID: clusterId,
								Op:        op,
								Resp:      resp,
							}
						}
					} else {
						var data *AnimusErrorResponse
						err := cbor.Unmarshal(payload, &data)

						if err != nil {
							data = &AnimusErrorResponse{
								Message: "client error: error unmarshaling payload: " + err.Error(),
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

					newCount := n.ResponseCount.Add(1)

					if n.ExpectedCount != 0 {
						if newCount > n.ExpectedCount {
							l.Warn("[animus magic] received more responses than expected", zap.String("commandId", commandId))
							if response.Op == OpResponse {
								c.UpdateCache(response) // At least update the cache
							}
							c.CloseNotifier(commandId) // Close the notifier
							return
						}
					}

					n.Chan <- response
					if response.Op == OpResponse {
						c.UpdateCache(response)
					}

					if n.ExpectedCount != 0 && newCount == n.ExpectedCount {
						c.CloseNotifier(commandId)
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

func (c *AnimusMagicClient) Publish(ctx context.Context, redis rueidis.Client, payload []byte) error {
	return redis.Do(ctx, redis.B().Publish().Channel(c.Channel).Message(rueidis.BinaryString(payload)).Build()).Error()
}

// RequestData stores the data for a request
type RequestOptions struct {
	ClusterID             *uint16 // must be set, also ExpectedResponseCount must be set if wildcard
	ExpectedResponseCount uint32  // must be set if wildcard. this is the number of responses expected
	CommandID             string  // if unset, will be randomly generated
	Scope                 byte    // if unset is ScopeBot
	Op                    byte    // if unset is OpRequest
	IgnoreOpError         bool    // if true, will ignore OpError responses
}

// Parse parses a RequestOptions
func (o *RequestOptions) Parse() error {
	if o == nil {
		return ErrNilRequestData
	}

	if o.ClusterID == nil {
		return ErrNilClusterID
	}

	if o.ExpectedResponseCount == 0 {
		if *o.ClusterID == WildcardClusterID {
			return ErrNilExpectedResponseCount
		} else {
			o.ExpectedResponseCount = 1
		}
	}

	if o.CommandID == "" {
		o.CommandID = crypto.RandString(16)
	}

	return nil
}

// CreateNotifier adds a notifier to the map and returns the channel
//
// This channel will receive the response for the given command id
func (c *AnimusMagicClient) CreateNotifier(opts *RequestOptions) chan *ClientResponse {
	// Create a channel to receive the response
	notify := make(chan *ClientResponse, opts.ExpectedResponseCount)

	// Store the channel in the notify map
	c.Notify.Store(opts.CommandID, &NotifyWrapper{
		Chan:          notify,
		ExpectedCount: opts.ExpectedResponseCount,
	})

	return notify
}

// CloseNotifier closes the notifier for the given command id
func (c *AnimusMagicClient) CloseNotifier(commandId string) {
	n, ok := c.Notify.Load(commandId)

	if !ok {
		return
	}

	c.Notify.Delete(commandId)
	close(n.Chan)
}

// GatherResponses gathers responses from the given notifier
//
// This waits for the expected number of responses or until the context is done
func (c *AnimusMagicClient) GatherResponses(
	ctx context.Context,
	opts *RequestOptions,
	notify chan *ClientResponse,
) (r []*ClientResponse, err error) {
	// Wait for the response
	var resps []*ClientResponse

	ercInt := int(opts.ExpectedResponseCount) // Cast beforehand to avoid casting every time

	for {
		select {
		case <-ctx.Done():
			// Remove the notifier
			c.CloseNotifier(opts.CommandID)
			return nil, ctx.Err()
		case resp := <-notify:
			resps = append(resps, resp)

			if resp.Op == OpError && !opts.IgnoreOpError {
				return resps, ErrOpError
			}

			if len(resps) >= ercInt {
				return resps, nil
			}
		}
	}
}

// RawRequest sends a raw request to the given cluster id and waits for a response
func (c *AnimusMagicClient) RawRequest(
	ctx context.Context,
	redis rueidis.Client,
	msg []byte,
	data *RequestOptions,
) ([]*ClientResponse, error) {
	if msg == nil {
		return nil, ErrNilMessage
	}

	if data == nil {
		return nil, ErrNilRequestData
	}

	err := data.Parse()

	if err != nil {
		return nil, err
	}

	// Create a channel to receive the response
	notify := c.CreateNotifier(data)

	// Publish the payload
	err = c.Publish(ctx, redis, msg)

	if err != nil {
		// Remove the notifier
		c.CloseNotifier(data.CommandID)
		return nil, err
	}

	// Wait for the response
	return c.GatherResponses(ctx, data, notify)
}

// Request sends a request to the given cluster id and waits for a response
func (c *AnimusMagicClient) Request(
	ctx context.Context,
	redis rueidis.Client,
	msg *AnimusMessage,
	data *RequestOptions,
) ([]*ClientResponse, error) {
	if msg == nil {
		return nil, ErrNilMessage
	}

	if data == nil {
		return nil, ErrNilRequestData
	}

	err := data.Parse()

	if err != nil {
		return nil, err
	}

	payload, err := c.CreatePayload(data.Scope, *data.ClusterID, data.Op, data.CommandID, msg)

	if err != nil {
		return nil, err
	}

	// Create a channel to receive the response
	notify := c.CreateNotifier(data)

	// Publish the payload
	err = c.Publish(ctx, redis, payload)

	if err != nil {
		// Remove the notifier
		c.CloseNotifier(data.CommandID)
		return nil, err
	}

	// Wait for the response
	return c.GatherResponses(ctx, data, notify)
}
