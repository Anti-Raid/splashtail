// Animus Magic is the internal redis IPC system for internal communications between the bot and the server
package animusmagic

import (
	"context"
	"errors"
	"fmt"
	"os"
	"strconv"
	"sync/atomic"
	"time"

	"github.com/anti-raid/splashtail/core/go.std/utils/syncmap"
	"github.com/infinitybotlist/eureka/crypto"
	"github.com/infinitybotlist/eureka/jsonimpl"
	"github.com/redis/rueidis"
	"go.uber.org/zap"
)

// Helper function to serialize data to the correct/current format
func SerializeData[T any](data T) ([]byte, error) {
	return jsonimpl.Marshal(data)
}

// Helper function to deserialize data from the correct/current format
func DeserializeData[T any](data []byte, d *T) error {
	return jsonimpl.Unmarshal(data, d)
}

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
	err := DeserializeData(c.RawPayload, &req)
	return &req, err
}

type NotifyWrapper struct {
	Chan          chan *ClientResponse
	ExpectedCount uint32
	ResponseCount atomic.Uint32
}

type AnimusMagicClient struct {
	// Target / the identity the client is for
	Identify AnimusTarget

	// The cluster id
	ClusterID uint16

	// On request function, if set, will be called upon recieving op of type OpRequest
	OnRequest func(*ClientRequest) (AnimusResponse, error)

	// On response function, if set, will be called upon recieving op of type OpResponse
	OnResponse func(*ClientResponse) error

	// Middleware function, will be called regardless of the op
	//
	// If bool is false, the message will be ignored/dropped for further processing
	OnMiddleware func(*AnimusMessageMetadata, []byte) (bool, error)

	// Allow all requests
	AllowAll bool

	// Set of notifiers
	Notify syncmap.Map[string, *NotifyWrapper]

	// The redis channel to use
	Channel string

	// The process id
	pid string
}

// New returns a new AnimusMagicClient
func New(channel string, identity AnimusTarget, clusterId uint16) *AnimusMagicClient {
	return &AnimusMagicClient{
		Channel:   channel,
		Identify:  identity,
		ClusterID: clusterId,
		pid:       strconv.Itoa(os.Getpid()),
	}
}

// ListenOnce starts listening for messages from redis
//
// This is *blocking* and should be run in a goroutine
func (c *AnimusMagicClient) ListenOnce(ctx context.Context, r rueidis.Client, l *zap.Logger) error {
	if c.pid == "" {
		panic("pid is not set")
	}
	return r.Dedicated(
		func(redis rueidis.DedicatedClient) error {
			return redis.Receive(ctx, redis.B().Subscribe().Channel(c.Channel).Build(), func(msg rueidis.PubSubMessage) {
				bytesData := []byte(msg.Message)

				meta, err := c.GetPayloadMeta(bytesData)

				if err != nil {
					l.Error("[animus magic] error getting payload metadata", zap.Error(err))
					return
				}

				// Filter message if required
				if !c.AllowAll {
					if !c.Filter(meta) {
						return
					}
				}

				go c.Handle(ctx, r, l, meta, bytesData[meta.PayloadOffset:])
			})
		},
	)
}

// Filter filters a message
func (c *AnimusMagicClient) Filter(meta *AnimusMessageMetadata) bool {
	// If the message is not to us, ignore it
	if meta.To != c.Identify && meta.To != AnimusTargetWildcard {
		return false
	}

	// If the target cluster id
	if meta.ClusterIDTo != c.ClusterID && meta.ClusterIDTo != WildcardClusterID {
		return false
	}

	return true
}

// Handle handles a message
func (c *AnimusMagicClient) Handle(ctx context.Context, redis rueidis.Client, l *zap.Logger, meta *AnimusMessageMetadata, payload []byte) {
	if c.OnMiddleware != nil {
		ok, err := c.OnMiddleware(meta, payload)

		if err != nil {
			l.Error("[animus magic] error in middleware", zap.Error(err))
			return
		}

		if !ok {
			return
		}
	}

	switch meta.Op {
	case OpProbe:
		cp, err := c.CreatePayload(
			c.Identify,
			meta.From,
			c.ClusterID,
			meta.ClusterIDFrom,
			OpResponse, // All Probes should use OpResponse
			meta.CommandID,
			c.pid,
		)

		if err != nil {
			l.Error("[animus magic] error creating error payload", zap.Error(err))
			return
		}

		err = c.Publish(ctx, redis, cp)

		if err != nil {
			l.Error("[animus magic] error publishing error payload", zap.Error(err))
			return
		}
	case OpRequest:
		if c.OnRequest != nil {
			resp, err := c.OnRequest(&ClientRequest{
				Meta:       meta,
				RawPayload: payload,
			})

			if err != nil {
				cp, err := c.CreatePayload(
					c.Identify,
					meta.From,
					c.ClusterID,
					meta.ClusterIDFrom,
					OpError,
					meta.CommandID,
					&AnimusErrorResponse{Message: err.Error(), Context: fmt.Sprint(resp)},
				)

				if err != nil {
					l.Error("[animus magic] error creating error payload", zap.Error(err))
					return
				}

				err = c.Publish(ctx, redis, cp)

				if err != nil {
					l.Error("[animus magic] error publishing error payload", zap.Error(err))
					return
				}
			} else if resp != nil {
				cp, err := c.CreatePayload(
					c.Identify,
					meta.From,
					c.ClusterID,
					meta.ClusterIDFrom,
					OpResponse,
					meta.CommandID,
					resp,
				)

				if err != nil {
					l.Error("[animus magic] error creating response payload", zap.Error(err))
					return
				}

				err = c.Publish(ctx, redis, cp)

				if err != nil {
					l.Error("[animus magic] error publishing response payload", zap.Error(err))
					return
				}
			}
		}
	case OpError:
		fallthrough // Both response and error are handled the same way
	case OpResponse:
		if c.OnResponse != nil {
			go func() {
				err := c.OnResponse(&ClientResponse{
					Meta:       meta,
					RawPayload: payload,
				})

				if err != nil {
					l.Error("[animus magic] error handling response", zap.Error(err))
				}
			}()
		}

		n, ok := c.Notify.Load(meta.CommandID)

		if !ok {
			if c.Identify == AnimusTargetInfra {
				return // Infra doesn't need this warning, its just spam when there is so much infra (wafflepaw, animuscli)
			}
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
			RawPayload: payload,
		}

		if n.ExpectedCount != 0 && newCount == n.ExpectedCount {
			c.CloseNotifier(meta.CommandID)
		}
	default:
		l.Warn("[animus magic] received unknown op", zap.Uint8("op", uint8(meta.Op)))
	}
}

// Listen starts listening for messages from redis
// and restarts the listener if it dies
func (c *AnimusMagicClient) Listen(ctx context.Context, redis rueidis.Client, l *zap.Logger) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			err := c.ListenOnce(ctx, redis, l)

			if errors.Is(err, context.Canceled) {
				return nil
			}

			if err != nil {
				l.Error("[animus magic] error listening to redis", zap.Error(err))
				time.Sleep(1 * time.Second)
			}
		}
	}
}

// CreatePayload creates a payload for the given command id and message
func (c *AnimusMagicClient) CreatePayload(
	from, to AnimusTarget,
	clusterIdFrom uint16,
	clusterIdTo uint16,
	op AnimusOp,
	commandId string,
	data any,
) ([]byte, error) {
	var finalPayload = []byte{
		byte(from),
		byte(to),
		byte(clusterIdFrom>>8) & 0xFF,
		byte(clusterIdFrom) & 0xFF,
		byte(clusterIdTo>>8) & 0xFF,
		byte(clusterIdTo) & 0xFF,
		byte(op),
	}

	finalPayload = append(finalPayload, []byte(commandId+"/")...)

	payload, err := SerializeData(data)

	if err != nil {
		return nil, err
	}

	finalPayload = append(finalPayload, payload...)

	return finalPayload, nil
}

// GetPayloadMeta parses the payload metadata from a message
func (c *AnimusMagicClient) GetPayloadMeta(payload []byte) (*AnimusMessageMetadata, error) {
	/*
			    const FROM_BYTE: usize = 0;
		    const TO_BYTE: usize = FROM_BYTE + 1;
		    const CLUSTER_ID_FROM_BYTE: usize = TO_BYTE + 1;
		    const CLUSTER_ID_TO_BYTE: usize = CLUSTER_ID_FROM_BYTE + 2;
		    const OP_BYTE: usize = CLUSTER_ID_TO_BYTE + 2;
	*/
	const FROM_BYTE = 0
	const TO_BYTE = FROM_BYTE + 1
	const CLUSTER_ID_FROM_BYTE = TO_BYTE + 1
	const CLUSTER_ID_TO_BYTE = CLUSTER_ID_FROM_BYTE + 2
	const OP_BYTE = CLUSTER_ID_TO_BYTE + 2

	if len(payload) < OP_BYTE {
		return nil, ErrInvalidPayload
	}

	meta := &AnimusMessageMetadata{
		From:          AnimusTarget(payload[FROM_BYTE]),
		To:            AnimusTarget(payload[TO_BYTE]),
		ClusterIDFrom: uint16(payload[CLUSTER_ID_FROM_BYTE])<<8 | uint16(payload[CLUSTER_ID_FROM_BYTE+1]),
		ClusterIDTo:   uint16(payload[CLUSTER_ID_TO_BYTE])<<8 | uint16(payload[CLUSTER_ID_TO_BYTE+1]),
		Op:            AnimusOp(payload[OP_BYTE]),
	}

	// Next bytes are the command id but only till '/'
	commandId := ""
	for i := OP_BYTE + 1; i < len(payload); i++ {
		if payload[i] == '/' {
			break
		}
		commandId += string(payload[i])
	}

	meta.CommandID = commandId
	meta.PayloadOffset = uint(len(commandId) + OP_BYTE + 2)

	return meta, nil
}

func (c *AnimusMagicClient) Publish(ctx context.Context, redis rueidis.Client, payload []byte) error {
	return redis.Do(ctx, redis.B().Publish().Channel(c.Channel).Message(rueidis.BinaryString(payload)).Build()).Error()
}

// RequestOptions stores the options for a request
type RequestOptions struct {
	ClusterID             *uint16      // the cluster id to send to, must be set, also ExpectedResponseCount must be set if wildcard
	ExpectedResponseCount uint32       // must be set if wildcard. this is the number of responses expected
	CommandID             string       // if unset, will be randomly generated
	To                    AnimusTarget // must be set
	Op                    AnimusOp     // must be set
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
		c.ClusterID,
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

// Parsed client response, contains the response and the error if any
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
		err := DeserializeData(cr.RawPayload, &errResp)

		if err != nil {
			return nil, err
		}

		return &ParsedClientResponse[T]{
			Err:        &errResp,
			ClientResp: cr,
		}, nil
	} else if cr.Meta.Op == OpResponse {
		var resp T
		err := DeserializeData(cr.RawPayload, &resp)

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
