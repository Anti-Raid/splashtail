// Animus Magic is the internal redis IPC system for internal communications between the bot and the server
//
// Format of payloads: <target [from]: u8><target [to]: u8><cluster id: u16><op: 8 bits><command id: alphanumeric string>/<cbor payload>
package animusmagic

import (
	"context"
	"errors"
	"fmt"
	"sync/atomic"
	"time"

	"github.com/anti-raid/splashtail/splashcore/utils/syncmap"
	"github.com/fxamacker/cbor/v2"
	"github.com/infinitybotlist/eureka/crypto"
	"github.com/redis/rueidis"
	"go.uber.org/zap"
)

// A ClientResponse contains the response from animus magic
type ClientResponse struct {
	Meta *AnimusMessageMetadata

	// The raw payload
	RawPayload []byte
}

// A ClientResponse contains the request from animus magic
type ClientRequest struct {
	Meta *AnimusMessageMetadata

	// The raw payload
	RawPayload []byte
}

func ParseClientRequest[T AnimusMessage](c *ClientRequest) (*T, error) {
	var req T
	err := cbor.Unmarshal(c.RawPayload, &req)
	return &req, err
}

type NotifyWrapper struct {
	Chan          chan *ClientResponse
	ExpectedCount uint32
	ResponseCount atomic.Uint32
}

type AnimusMagicClient struct {
	// Target / who the client is for
	From AnimusTarget

	// On request function, if set, will be called upon recieving op of type OpRequest
	OnRequest func(*ClientRequest) (AnimusResponse, error)

	// Set of notifiers
	Notify syncmap.Map[string, *NotifyWrapper]

	// The redis channel to use
	Channel string
}

// New returns a new AnimusMagicClient
func New(channel string, from AnimusTarget) *AnimusMagicClient {
	return &AnimusMagicClient{
		Channel: channel,
		From:    from,
	}
}

// ListenOnce starts listening for messages from redis
//
// This is *blocking* and should be run in a goroutine
func (c *AnimusMagicClient) ListenOnce(ctx context.Context, r rueidis.Client, l *zap.Logger) error {
	return r.Dedicated(
		func(redis rueidis.DedicatedClient) error {
			return redis.Receive(ctx, redis.B().Subscribe().Channel(c.Channel).Build(), func(msg rueidis.PubSubMessage) {
				bytesData := []byte(msg.Message)

				meta, err := c.GetPayloadMeta(bytesData)

				if err != nil {
					l.Error("[animus magic] error getting payload metadata", zap.Error(err))
					return
				}

				// If the target of the message is not us, ignore it
				if meta.To != c.From {
					return
				}

				go func() {
					switch meta.Op {
					case OpRequest:
						if c.OnRequest != nil {
							resp, err := c.OnRequest(&ClientRequest{
								Meta:       meta,
								RawPayload: bytesData[meta.PayloadOffset:],
							})

							if err != nil {
								cp, err := c.CreatePayload(
									c.From,
									meta.From,
									meta.ClusterID,
									OpError,
									meta.CommandID,
									&AnimusErrorResponse{Message: err.Error(), Context: fmt.Sprint(resp)},
								)

								if err != nil {
									l.Error("[animus magic] error creating error payload", zap.Error(err))
									return
								}

								err = c.Publish(ctx, r, cp)

								if err != nil {
									l.Error("[animus magic] error publishing error payload", zap.Error(err))
									return
								}
							} else if resp != nil {
								cp, err := c.CreatePayload(
									c.From,
									meta.From,
									meta.ClusterID,
									OpResponse,
									meta.CommandID,
									resp,
								)

								if err != nil {
									l.Error("[animus magic] error creating response payload", zap.Error(err))
									return
								}

								err = c.Publish(ctx, r, cp)

								if err != nil {
									l.Error("[animus magic] error publishing response payload", zap.Error(err))
									return
								}
							}
						}
					case OpError:
						fallthrough // Both response and error are handled the same way
					case OpResponse:
						if meta.From == AnimusTargetWebserver {
							return
						}

						n, ok := c.Notify.Load(meta.CommandID)

						if !ok {
							l.Warn("[animus magic] received response for unknown command", zap.String("commandId", meta.CommandID))
							return
						}

						newCount := n.ResponseCount.Add(1)

						if n.ExpectedCount != 0 {
							if newCount > n.ExpectedCount {
								l.Warn("[animus magic] received more responses than expected", zap.String("commandId", meta.CommandID))
								c.CloseNotifier(meta.CommandID) // Close the notifier
								return
							}
						}

						n.Chan <- &ClientResponse{
							Meta:       meta,
							RawPayload: bytesData[meta.PayloadOffset:],
						}

						if n.ExpectedCount != 0 && newCount == n.ExpectedCount {
							c.CloseNotifier(meta.CommandID)
						}
					default:
						l.Warn("[animus magic] received unknown op", zap.Uint8("op", uint8(meta.Op)))
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
func (c *AnimusMagicClient) CreatePayload(from, to AnimusTarget, clusterId uint16, op AnimusOp, commandId string, resp any) ([]byte, error) {
	var finalPayload = []byte{
		byte(from),
		byte(to),
		byte(clusterId>>8) & 0xFF,
		byte(clusterId) & 0xFF,
		byte(op),
	}

	finalPayload = append(finalPayload, []byte(commandId+"/")...)

	payload, err := cbor.Marshal(resp)

	if err != nil {
		return nil, err
	}

	finalPayload = append(finalPayload, payload...)

	return finalPayload, nil
}

// GetPayloadMeta parses the payload metadata from a message
func (c *AnimusMagicClient) GetPayloadMeta(payload []byte) (*AnimusMessageMetadata, error) {
	if len(payload) < 6 {
		return nil, ErrInvalidPayload
	}

	meta := &AnimusMessageMetadata{
		From:      AnimusTarget(payload[0]),
		To:        AnimusTarget(payload[1]),
		ClusterID: uint16(payload[2])<<8 | uint16(payload[3]),
		Op:        AnimusOp(payload[4]),
	}

	// Next bytes are the command id but only till '/'
	commandId := ""
	for i := 5; i < len(payload); i++ {
		if payload[i] == '/' {
			break
		}
		commandId += string(payload[i])
	}

	meta.CommandID = commandId
	meta.PayloadOffset = uint(len(commandId) + 6)

	return meta, nil
}

func (c *AnimusMagicClient) Publish(ctx context.Context, redis rueidis.Client, payload []byte) error {
	return redis.Do(ctx, redis.B().Publish().Channel(c.Channel).Message(rueidis.BinaryString(payload)).Build()).Error()
}

// RequestOptions stores the data for a request
type RequestOptions struct {
	ClusterID             *uint16      // must be set, also ExpectedResponseCount must be set if wildcard
	ExpectedResponseCount uint32       // must be set if wildcard. this is the number of responses expected
	CommandID             string       // if unset, will be randomly generated
	To                    AnimusTarget // if unset, is set to AnimusTargetBot
	Op                    AnimusOp     // if unset is OpRequest
	IgnoreOpError         bool         // if true, will ignore OpError responses
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
		o.CommandID = NewCommandId()
	}

	return nil
}

// CreateNotifier adds a notifier to the map and returns the channel
//
// This channel will receive the response for the given command id
func (c *AnimusMagicClient) CreateNotifier(commandId string, expectedResponseCount uint32) chan *ClientResponse {
	// Create a channel to receive the response
	notify := make(chan *ClientResponse, expectedResponseCount)

	// Store the channel in the notify map
	c.Notify.Store(commandId, &NotifyWrapper{
		Chan:          notify,
		ExpectedCount: expectedResponseCount,
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

			if resp.Meta.Op == OpError && !opts.IgnoreOpError {
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
	notify := c.CreateNotifier(data.CommandID, data.ExpectedResponseCount)

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
	msg AnimusMessage,
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

	if msg.Target() != data.To {
		return nil, ErrInvalidTarget
	}

	payload, err := c.CreatePayload(
		AnimusTargetWebserver,
		data.To,
		*data.ClusterID,
		data.Op,
		data.CommandID,
		msg,
	)

	if err != nil {
		return nil, err
	}

	// Create a channel to receive the response
	notify := c.CreateNotifier(data.CommandID, data.ExpectedResponseCount)

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

type ParsedClientResponse[T AnimusResponse] struct {
	Err        *AnimusErrorResponse
	Resp       *T
	ClientResp *ClientResponse
}

func ParseClientResponse[T AnimusResponse](
	cr *ClientResponse,
) (*ParsedClientResponse[T], error) {
	if cr.Meta.Op == OpError {
		var errResp AnimusErrorResponse
		err := cbor.Unmarshal(cr.RawPayload, &errResp)

		if err != nil {
			return nil, err
		}

		return &ParsedClientResponse[T]{
			Err:        &errResp,
			ClientResp: cr,
		}, nil
	} else if cr.Meta.Op == OpResponse {
		var resp T
		err := cbor.Unmarshal(cr.RawPayload, &resp)

		if err != nil {
			return nil, err
		}

		return &ParsedClientResponse[T]{
			Resp:       &resp,
			ClientResp: cr,
		}, nil
	} else {
		return nil, errors.ErrUnsupported
	}
}

func ParseClientResponses[T AnimusResponse](
	cr []*ClientResponse,
) ([]*ParsedClientResponse[T], error) {
	var resp []*ParsedClientResponse[T]

	for _, r := range cr {
		p, err := ParseClientResponse[T](r)

		if err != nil {
			return nil, err
		}

		resp = append(resp, p)
	}

	return resp, nil
}

// Helper function to create a new command id
func NewCommandId() string {
	return crypto.RandString(16)
}
