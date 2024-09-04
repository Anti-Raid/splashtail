package core

import (
	"bytes"
	"catway/core/sandwichjson"
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"net"
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/WelcomerTeam/czlib"
	"github.com/disgoorg/disgo/discord"
	"github.com/disgoorg/disgo/gateway"
	"github.com/disgoorg/snowflake/v2"
	"nhooyr.io/websocket"
)

const WebsocketReadLimit = 512 << 20

var (
	heartbeatAck = []byte(`{"op":11}`)
)

// Returns first match from a map and handles keys as non case sensitive.
func GetEntry(m map[string]interface{}, key string) interface{} {
	key = strings.ToLower(key)
	for i, k := range m {
		if strings.ToLower(i) == key {
			return k
		}
	}

	return nil
}

// chatServer enables broadcasting to a set of subscribers.
type chatServer struct {

	// catway
	catway *Catway

	subscribers map[[2]int32][]*subscriber
	// the expected token
	expectedToken string

	// external address (used for resuming)
	externalAddress string

	// address
	address string

	// serveMux routes the various endpoints to the appropriate handler.
	serveMux http.ServeMux

	// defaultWriteDelay
	defaultWriteDelay int64

	// subscriberMessageBuffer controls the max number
	// of messages that can be queued for a subscriber
	// before it is kicked.
	//
	// Defaults to 100000.
	subscriberMessageBuffer int

	subscribersMu sync.RWMutex
}

// newChatServer constructs a chatServer with the defaults.
func newChatServer() *chatServer {
	cs := &chatServer{
		subscribers: make(map[[2]int32][]*subscriber),
	}
	cs.serveMux.HandleFunc("/", cs.subscribeHandler)
	cs.serveMux.HandleFunc("/publish", cs.publishHandler)

	return cs
}

type message struct {
	// What message to send, note that sequence will be automatically set
	message *CatwayPayload
	// close string, will be sent on close
	closeString string
	// What raw bytes to send, this bypasses seq additions etc.
	rawBytes []byte
	// close code, if set will close the connection
	closeCode websocket.StatusCode
}

// subscriber represents a subscriber.
// Messages are sent on the msgs channel and if the client
// cannot keep up with the messages, closeSlow is called.
type subscriber struct {
	c              *websocket.Conn
	cancelFunc     context.CancelFunc
	reader         chan *CatwayPayload
	writer         chan message
	writeHeartbeat chan void
	sessionId      string
	writeDelay     int64
	shard          [2]int32
	seq            int
	up             bool
	resumed        bool
	moving         bool
}

// invalidSession closes the connection with the given reason.
func (cs *chatServer) invalidSession(s *subscriber, reason string, resumable bool) {
	cs.catway.Logger.Error("[WS] Invalid session: %s, is resumable: %v", reason, resumable)

	if resumable {
		s.writer <- message{
			rawBytes:    []byte(`{"op":9,"d":true}`),
			closeCode:   websocket.StatusCode(4000),
			closeString: "Invalid Session",
		}
	} else {
		s.writer <- message{
			rawBytes:    []byte(`{"op":9,"d":false}`),
			closeCode:   websocket.StatusCode(4000),
			closeString: "Invalid Session",
		}
	}
}

func (cs *chatServer) dispatchInitial(done chan void, s *subscriber) error {
	user, ok := cs.catway.Discord.Caches().SelfUser()

	if !ok {
		return errors.New("failed to get self user")
	}

	cs.catway.Logger.Info("[WS] Shard %d/%d (now dispatching events) %v", slog.Int("shardId", int(s.shard[0])))

	// Get all guilds
	var guildIdShardIdMap = make(map[snowflake.ID]int32)

	unavailableGuilds := make([]discord.UnavailableGuild, 0)

	cs.catway.Discord.Caches().GuildCache().ForEach(func(g discord.Guild) {
		shardId := int32(cs.catway.GetShardIdOfGuild(g.ID, cs.catway.ConsumerShardCount()))
		guildIdShardIdMap[g.ID] = shardId // We need this when dispatching guilds
		if shardId == s.shard[0] {
			unavailableGuilds = append(unavailableGuilds, discord.UnavailableGuild{
				ID:          g.ID,
				Unavailable: false,
			})
		}
	})

	// First send READY event with our initial state
	readyPayload := map[string]any{
		"v":          10,
		"user":       user,
		"session_id": s.sessionId,
		"shard":      []int32{s.shard[0], s.shard[1]},
		"application": map[string]any{
			"id":    user.ID,
			"flags": int32(user.Flags),
		},
		"resume_gateway_url": cs.externalAddress,
		"guilds":             unavailableGuilds,
	}

	select {
	case <-done:
		return nil
	default:
	}

	serializedReadyPayload, err := sandwichjson.Marshal(readyPayload)

	if err != nil {
		cs.catway.Logger.Error("[WS] Failed to marshal ready payload", slog.String("err", err.Error()))
		return err
	}

	cs.catway.Logger.Info("[WS] Dispatching ready to shard", slog.Int("shardId", int(s.shard[0])))

	s.writer <- message{
		message: &CatwayPayload{
			Op:   gateway.OpcodeDispatch,
			Data: serializedReadyPayload,
			Type: "READY",
		},
	}

	// Next dispatch guilds
	cs.catway.Discord.Caches().GuildCache().ForEach(func(g discord.Guild) {
		select {
		case <-done:
			return
		default:
		}

		shardId, ok := guildIdShardIdMap[g.ID]

		if !ok {
			// Get shard id
			shardId = int32(cs.catway.GetShardIdOfGuild(g.ID, cs.catway.ConsumerShardCount()))
		}

		if shardId != s.shard[0] {
			return // Skip to next guild if the shard id is not the same
		}

		if g.AfkChannelID == nil {
			g.AfkChannelID = &g.ID
		}

		serializedGuild, err := sandwichjson.Marshal(g)

		if err != nil {
			cs.catway.Logger.Error("[WS] Failed to marshal guild: %s [shard %d]", err.Error(), s.shard[0])
			return
		}

		s.writer <- message{
			message: &CatwayPayload{
				Op:   gateway.OpcodeDispatch,
				Data: serializedGuild,
				Type: "GUILD_CREATE",
			},
		}
	})

	cs.catway.Logger.Info("[WS] Shard %d (initial state dispatched successfully)", s.shard[0])

	return err
}

func (cs *chatServer) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	cs.serveMux.ServeHTTP(w, r)
}

// subscribeHandler accepts the WebSocket connection and then subscribes
// it to all future messages.
func (cs *chatServer) subscribeHandler(w http.ResponseWriter, r *http.Request) {
	// Check for special query params
	//
	// - writeDelay (int): the delay to write messages in microseconds
	var writeDelay int64

	wd := r.URL.Query().Get("writeDelay")

	cs.catway.Logger.Info("[WS] Shard %d is now subscribing", 0)

	if wd != "" {
		// Parse to int
		delay, err := strconv.ParseInt(wd, 10, 64)

		if err != nil {
			http.Error(w, "Invalid writeDelay", http.StatusBadRequest)
			return
		}

		writeDelay = delay
	} else {
		writeDelay = cs.defaultWriteDelay
	}

	cs.subscribe(r.Context(), w, r, writeDelay)
}

// publishHandler reads the request body with a limit of 8192 bytes and then publishes
// the received message.
func (cs *chatServer) publishHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != "POST" {
		http.Error(w, http.StatusText(http.StatusMethodNotAllowed), http.StatusMethodNotAllowed)
		return
	}

	// Get shard from query params
	var shard [2]int32

	shardStr := r.URL.Query().Get("shard")

	if shardStr != "" {
		_, err := fmt.Sscanf(shardStr, "%d-%d", &shard[0], &shard[1])
		if err != nil {
			http.Error(w, http.StatusText(http.StatusBadRequest), http.StatusBadRequest)
			return
		}
	}

	body := http.MaxBytesReader(w, r.Body, 8192)
	msg, err := io.ReadAll(body)
	if err != nil {
		http.Error(w, http.StatusText(http.StatusRequestEntityTooLarge), http.StatusRequestEntityTooLarge)
		return
	}

	var payload CatwayPayload

	err = sandwichjson.Unmarshal(msg, &payload)
	if err != nil {
		http.Error(w, http.StatusText(http.StatusBadRequest), http.StatusBadRequest)
		return
	}

	cs.publish(shard, &payload)

	w.WriteHeader(http.StatusAccepted)
}

// identifyClient tries to identify a incoming connection
func (cs *chatServer) identifyClient(done chan void, s *subscriber) (oldSess *subscriber, err error) {
	// Before adding the subscriber for external access, send the initial hello payload and wait for identify
	// If the client does not identify within 5 seconds, close the connection
	s.writer <- message{
		rawBytes: []byte(`{"op":10,"d":{"heartbeat_interval":41250}}`),
	}

	// Keep reading messages till we reach an identify
	for {
		select {
		case <-done:
			return nil, nil
		case <-time.After(5 * time.Second):
			return nil, errors.New("timed out waiting for identify")
		case packet := <-s.reader:
			// Read an identify packet
			//
			// Note that resume is not supported at this time
			if packet.Op == gateway.OpcodeIdentify {
				var identify struct {
					Token string   `json:"token"`
					Shard [2]int32 `json:"shard"`
				}

				err := sandwichjson.Unmarshal(packet.Data, &identify)
				if err != nil {
					return nil, fmt.Errorf("failed to unmarshal identify packet: %w", err)
				}

				if len(identify.Shard) != 2 {
					return nil, errors.New("invalid shard")
				}

				identify.Token = strings.Replace(identify.Token, "Bot ", "", 1)

				if identify.Token != cs.expectedToken {
					return nil, errors.New("invalid token")
				}

				s.sessionId = randomHex(12)

				csc := cs.catway.ConsumerShardCount() // Get the consumer shard count to avoid unneeded casts

				// dpy workaround
				if identify.Shard[1] == 0 {
					identify.Shard[1] = csc
				}

				if identify.Shard[1] > csc {
					return nil, fmt.Errorf("invalid shard count: %d > %d", identify.Shard[1], csc)
				} else if identify.Shard[0] > csc {
					return nil, fmt.Errorf("invalid shard id: %d > %d", identify.Shard[0], csc)
				}

				s.shard = identify.Shard
				s.up = true

				cs.catway.Logger.Info("[WS] Shard %d is now identified with created session id %s [%s]", s.shard[0], s.sessionId, fmt.Sprint(s.shard))
				return nil, nil
			} else if packet.Op == gateway.OpcodeResume {
				var resume struct {
					Token     string `json:"token"`
					SessionID string `json:"session_id"`
					Seq       int    `json:"seq"`
				}

				err := sandwichjson.Unmarshal(packet.Data, &resume)
				if err != nil {
					return nil, fmt.Errorf("failed to unmarshal resume packet: %w", err)
				}

				resume.Token = strings.Replace(resume.Token, "Bot ", "", 1)

				if resume.Token == cs.expectedToken {
					// Find session with same session id
					cs.subscribersMu.RLock()
					for _, shardSubs := range cs.subscribers {
						for _, oldSess := range shardSubs {
							if s.sessionId == resume.SessionID {
								cs.catway.Logger.Info("[WS] Shard %d is now identified with resumed session id %s [%s]", s.shard[0], s.sessionId, fmt.Sprint(s.shard))
								s.seq = resume.Seq
								s.shard = oldSess.shard
								s.resumed = true
								s.up = true
								cs.subscribersMu.RUnlock()
								return oldSess, nil
							}
						}
					}
					cs.subscribersMu.RUnlock()

					if !s.up {
						return nil, errors.New("invalid session id")
					}
				} else {
					return nil, errors.New("invalid token")
				}
			}
		}
	}
}

// reader reads messages from subscribe and sends them to the reader
// Note that there must be only one reader reading from the goroutine
func (cs *chatServer) readMessages(done chan void, s *subscriber) {
	ctx, cancelFunc := context.WithCancel(context.Background())

	defer func() {
		cancelFunc()
		s.cancelFunc()

		if err := recover(); err != nil {
			cs.catway.Logger.Error("[WS] Shard %d panicked on readMessages: %v", s.shard[0], err)
			cs.invalidSession(s, "panicked", true)
			return
		}
	}()

	for {
		select {
		case <-done:
			return
		default:
			typ, ior, err := s.c.Read(ctx)

			if err != nil {
				return
			}

			select {
			case <-ctx.Done():
				return
			default:
			}

			var payload *CatwayPayload
			switch typ {
			case websocket.MessageText:
				err := sandwichjson.Unmarshal(ior, &payload)

				if err != nil {
					cs.catway.Logger.Error("[WS] Failed to unmarshal packet: %s", err.Error())
					cs.invalidSession(s, "failed to unmarshal packet: "+err.Error(), true)
					return
				}
			case websocket.MessageBinary:
				// ZLIB compressed message sigh
				newReader, err := czlib.NewReader(bytes.NewReader(ior))

				if err != nil {
					cs.catway.Logger.Error("[WS] Failed to decompress message: %s", err.Error())
					cs.invalidSession(s, "failed to decompress message: "+err.Error(), true)
					return
				}

				err = sandwichjson.UnmarshalReader(newReader, &payload)

				if err != nil {
					cs.catway.Logger.Error("[WS] Failed to unmarshal packet: %s", err.Error())
					cs.invalidSession(s, "failed to unmarshal packet: "+err.Error(), true)
					return
				}
			}

			if payload.Op == gateway.OpcodeHeartbeat {
				s.writeHeartbeat <- struct{}{}
			} else {
				s.reader <- payload
			}
		}
	}
}

// handleReadMessages handles messages from reader
func (cs *chatServer) handleReadMessages(done chan void, s *subscriber) {
	for {
		select {
		case <-done:
			return
		case msg := <-s.reader:
			// Send to discord directly
			cs.catway.Logger.Debug("[WS] Shard %d got/found packet: %v %s", s.shard[0], msg, string(msg.Data))

			// Try finding guild_id
			var shardId = s.shard[0]
			var gwShardCount = int32(cs.catway.ShardCount())
			if s.shard[1] != gwShardCount {
				cs.catway.Logger.Info("Shard %d is not using global shard count, remapping to real shard for read message %v", s.shard[0], msg)

				var guildId struct {
					GuildID snowflake.ID `json:"guild_id"`
				}

				err := sandwichjson.Unmarshal(msg.Data, &guildId)

				if err != nil || guildId.GuildID == 0 {
					cs.catway.Logger.Info("No guild_id found in recieved packet %s", msg.Data)
					continue
				}

				shardId = int32(cs.catway.GetShardIdOfGuild(guildId.GuildID, gwShardCount))
				cs.catway.Logger.Info("Remapped shard id %d to %d", s.shard[0], shardId)
			}

			// Find the shard corresponding to the guild_id
			s := cs.catway.Discord.ShardManager().Shard(int(shardId))

			if s == nil {
				cs.catway.Logger.Error("[WS] Shard %d not found", shardId)
				continue
			}

			err := s.Send(cs.catway.Context, msg.Op, gateway.MessageDataUnknown(msg.Data))

			if err != nil {
				cs.catway.Logger.Error("[WS] Failed to send event: %s", err.Error())
			}
		}
	}
}

// writeMessages reads messages from the writer and sends them to the WebSocket
func (cs *chatServer) writeMessages(done chan void, s *subscriber) {
	ctx, cancelFunc := context.WithCancel(context.Background())
	defer func() {
		s.cancelFunc()
		cancelFunc()

		if err := recover(); err != nil {
			cs.catway.Logger.Error("[WS] Shard %d panicked on writeMessages: %v", s.shard[0], err)
			cs.invalidSession(s, "panicked", true)
			return
		}
	}()

	for {
		select {
		// Case 1: Done is closed
		case <-done:
			return
		// Case 2: Heartbeat
		case <-s.writeHeartbeat:
			if s.writeDelay > 0 {
				time.Sleep(time.Duration(s.writeDelay) * time.Microsecond)
			}

			err := s.c.Write(ctx, websocket.MessageText, heartbeatAck)

			if err != nil {
				cs.catway.Logger.Error("[WS] Failed to write heartbeat: %s", err.Error())
				return
			}
		// Case 3: Message is received
		case msg := <-s.writer:
			if s.writeDelay > 0 {
				time.Sleep(time.Duration(s.writeDelay) * time.Microsecond)
			}

			if len(msg.rawBytes) > 0 {
				err := s.c.Write(ctx, websocket.MessageText, msg.rawBytes)

				if err != nil {
					cs.catway.Logger.Error("[WS] Failed to write message [rawBytes]: %s", err.Error())
					s.c.Close(websocket.StatusInternalError, "Failed to write message [rawBytes]")
					return
				}
			}

			if msg.message != nil {
				if msg.message.Op == gateway.OpcodeDispatch {
					msg.message.Sequence = s.seq
					s.seq++
				} else {
					msg.message.Sequence = 0
				}

				serializedMessage, err := sandwichjson.Marshal(msg.message)

				if err != nil {
					cs.catway.Logger.Error("[WS] Failed to marshal message: %s", slog.String("err", err.Error()))
					continue
				}

				err = s.c.Write(ctx, websocket.MessageText, serializedMessage)

				if err != nil {
					cs.catway.Logger.Error("[WS] Failed to write message [serialized]: %s", slog.String("err", err.Error()))
					s.c.Close(websocket.StatusInternalError, "Failed to write message [serialized]")
					return
				}
			}

			if msg.closeCode != 0 {
				s.up = false
				s.cancelFunc()
				s.c.Close(msg.closeCode, msg.closeString)
				return
			}
		}
	}
}

// subscribe subscribes the given WebSocket to all broadcast messages.
// It creates a subscriber with a buffered msgs chan to give some room to slower
// connections and then registers the subscriber. It then listens for all messages
// and writes them to the WebSocket. If the context is cancelled or
// an error occurs, it returns and deletes the subscription.
//
// It uses CloseRead to keep reading from the connection to process control
// messages and cancel the context if the connection drops.
func (cs *chatServer) subscribe(ctx context.Context, w http.ResponseWriter, r *http.Request, writeDelay int64) error {
	var c *websocket.Conn
	s := &subscriber{
		reader:         make(chan *CatwayPayload, cs.subscriberMessageBuffer),
		writer:         make(chan message, cs.subscriberMessageBuffer),
		writeHeartbeat: make(chan void, cs.subscriberMessageBuffer),
		writeDelay:     writeDelay,
	}

	// Create cancellable ctx
	ctx, cancelFunc := context.WithCancel(ctx)
	s.cancelFunc = cancelFunc

	var err error
	c, err = websocket.Accept(w, r, nil)

	if err != nil {
		return err
	}

	c.SetReadLimit(WebsocketReadLimit)

	s.c = c

	defer c.Close(websocket.StatusCode(4000), `{"op":9,"d":true}`)

	// Create done channels for subscriber/reader/writer/handleReadMessages, allowing
	// a goroutine to then close
	writerDone := make(chan void)
	readerDone := make(chan void)
	identifyClientDone := make(chan void)
	dispatchInitialDone := make(chan void)
	handleReadMessagesDone := make(chan void)
	subscriberDone := make(chan void)

	go func() {
		<-ctx.Done()
		close(writerDone)
		close(readerDone)
		close(identifyClientDone)
		close(dispatchInitialDone)
		close(handleReadMessagesDone)
		close(subscriberDone)
	}()

	// Start the reader+writer bit
	go cs.writeMessages(writerDone, s)
	time.Sleep(1 * time.Millisecond)
	go cs.readMessages(readerDone, s)
	time.Sleep(1 * time.Millisecond)

	cs.catway.Logger.Info("[WS] Shard %d is now launched (reader+writer UP)", s.shard[0])

	// Now identifyClient
	oldSess, err := cs.identifyClient(identifyClientDone, s)

	if err != nil {
		cs.invalidSession(s, err.Error(), false)
		return err
	}

	cs.addSubscriber(s, s.shard)
	defer func() {
		// Give one minute for resumes
		time.Sleep(1 * time.Minute)
		cs.deleteSubscriber(s)
	}()

	// SAFETY: There should be no other reader at this point, so start up handleReadMessages
	go cs.handleReadMessages(handleReadMessagesDone, s)

	if oldSess != nil {
		cs.invalidSession(oldSess, "New session identified", true)
		oldSess.moving = true

		// Close old session
		cs.deleteSubscriber(oldSess)

		for msg := range oldSess.writer {
			select {
			case <-ctx.Done():
				return ctx.Err()
			default:
			}

			if msg.message == nil || msg.message.Op != gateway.OpcodeDispatch {
				continue
			}

			s.writer <- msg
		}
	}

	cs.catway.Logger.Info("[WS] Shard %d is now connected (oldSess fanout done)", s.shard[0])

	if !s.resumed {
		cs.dispatchInitial(dispatchInitialDone, s)
	} else {
		// Send a RESUMED event
		s.writer <- message{
			message: &CatwayPayload{
				Op:   gateway.OpcodeDispatch,
				Data: []byte(`{}`),
				Type: "RESUMED",
			},
		}
	}

	// Wait for the context to be cancelled
	// readMessages and writeMessages will handle the rest
	<-subscriberDone
	cs.catway.Logger.Info("[WS] Shard is now disconnected", slog.Int("shardId", int(s.shard[0])))
	return nil
}

// publish publishes the msg to all subscribers.
// It never blocks and so messages to slow subscribers
// are dropped.
func (cs *chatServer) publish(shard [2]int32, msg *CatwayPayload) {
	cs.catway.Logger.Debug("[WS] Shard is now publishing message", slog.Int("shardId", int(shard[0])))

	cs.subscribersMu.RLock()
	defer cs.subscribersMu.RUnlock()

	for subShard, sub := range cs.subscribers {
		if subShard[1] != shard[1] && msg.EventDispatchIdentifier.GuildID != nil {
			if subShard[1] <= 0 {
				// 0 shards is impossible, close the connection
				for _, s := range sub {
					s.writer <- message{
						rawBytes:    []byte(`{"op":9,"d":false}`),
						closeCode:   websocket.StatusCode(4000),
						closeString: "Invalid Session",
					}
				}
				continue
			}

			// Shard count used by subscriber is not the same as the shard count used by the message
			// We need to remap the shard id based on the subscriber's shard id
			msgShardId := cs.catway.GetShardIdOfGuild(*msg.EventDispatchIdentifier.GuildID, subShard[1])

			if msgShardId != shard[0] {
				continue // Skip if the remapped shard id is not the same
			}
		} else if subShard[0] != shard[0] {
			continue // Skip if the shard id is not the same
		}

		for _, s := range sub {
			if !s.up {
				continue
			}

			cs.catway.Logger.Debug("[WS] Shard is now publishing message to subscribers", slog.Int("shardId", int(shard[0])), slog.Int("subscriberCount", len(sub)))

			s.writer <- message{
				message: msg,
			}
		}
	}
}

// publishGlobal publishes the msg to all subscribers.
// It never blocks and so messages to slow subscribers
// are dropped.
func (cs *chatServer) publishGlobal(msg *CatwayPayload) {
	cs.catway.Logger.Debug("[WS] Global is now publishing message")

	cs.subscribersMu.RLock()
	defer cs.subscribersMu.RUnlock()

	for _, shardSubs := range cs.subscribers {
		for _, s := range shardSubs {
			if !s.up {
				continue
			}

			cs.catway.Logger.Debug("[WS] Global is now publishing message to %d subscribers", len(shardSubs))

			s.writer <- message{
				message: msg,
			}
		}
	}
}

// addSubscriber registers a subscriber.
func (cs *chatServer) addSubscriber(s *subscriber, shard [2]int32) {
	cs.subscribersMu.Lock()
	defer cs.subscribersMu.Unlock()

	if subs, ok := cs.subscribers[shard]; ok {
		cs.subscribers[shard] = append(subs, s)
	} else {
		cs.subscribers[shard] = []*subscriber{s}
	}
}

// deleteSubscriber deletes the given subscriber.
func (cs *chatServer) deleteSubscriber(s *subscriber) {
	cs.subscribersMu.Lock()
	defer cs.subscribersMu.Unlock()

	if sub, ok := cs.subscribers[s.shard]; ok {
		for i, is := range sub {
			if is.sessionId == s.sessionId || is == s {
				is.cancelFunc()
				cs.subscribers[s.shard] = append(sub[:i], sub[i+1:]...)
			}
		}
	}
}

type WebsocketClient struct {
	cs *chatServer
}

func (mg *WebsocketClient) String() string {
	return "websocket"
}

func (mq *WebsocketClient) Channel() string {
	return "websocket"
}

func (mq *WebsocketClient) Cluster() string {
	return "websocket"
}

// Supported options:
//
// address (string): the address to listen on
// expectedToken (string): the expected token for identify
// externalAddress (string): the external address to use for resuming, defaults to ws://address if unset
// defaultWriteDelay (int): the default write delay in microseconds, defaults to 10
func (mq *WebsocketClient) Connect(ctx context.Context, manager *Catway, clientName string, args map[string]interface{}) error {
	var ok bool

	var address string
	var externalAddress string
	var expectedToken string

	if address, ok = GetEntry(args, "Address").(string); !ok {
		return errors.New("websocketMQ connect: string type assertion failed for Address")
	}

	externalAddress, ok = GetEntry(args, "ExternalAddress").(string)

	if !ok {
		if !strings.HasPrefix(address, "ws") {
			externalAddress = "ws://" + address
		} else {
			externalAddress = address
		}
	}

	if expectedToken, ok = GetEntry(args, "ExpectedToken").(string); !ok {
		return errors.New("websocketMQ connect: string type assertion failed for ExpectedToken")
	}

	l, err := net.Listen("tcp", address)
	if err != nil {
		return errors.New("websocketMQ listen: " + err.Error())
	}

	mq.cs = newChatServer()
	mq.cs.expectedToken = expectedToken
	mq.cs.catway = manager
	mq.cs.address = address
	mq.cs.externalAddress = externalAddress
	s := &http.Server{Handler: mq.cs}

	switch defaultWriteDelay := GetEntry(args, "DefaultWriteDelay").(type) {
	case int:
		mq.cs.defaultWriteDelay = int64(defaultWriteDelay)
	case int64:
		mq.cs.defaultWriteDelay = defaultWriteDelay
	case float64:
		mq.cs.defaultWriteDelay = int64(defaultWriteDelay)
	case string:
		delay, err := strconv.ParseInt(defaultWriteDelay, 10, 64)

		if err != nil {
			return errors.New("websocketMQ connect: failed to parse DefaultWriteDelay: " + err.Error())
		}

		mq.cs.defaultWriteDelay = delay
	default:
		manager.Logger.Warn("DefaultWriteDelay not set, defaulting to 10 microseconds")
		mq.cs.defaultWriteDelay = 10
	}

	switch subscriberMessageBuffer := GetEntry(args, "SubscriberMessageBuffer").(type) {
	case int:
		mq.cs.subscriberMessageBuffer = subscriberMessageBuffer
	case int64:
		mq.cs.subscriberMessageBuffer = int(subscriberMessageBuffer)
	case float64:
		mq.cs.subscriberMessageBuffer = int(subscriberMessageBuffer)
	case string:
		buffer, err := strconv.ParseInt(subscriberMessageBuffer, 10, 64)

		if err != nil {
			return errors.New("websocketMQ connect: failed to parse SubscriberMessageBuffer: " + err.Error())
		}

		mq.cs.subscriberMessageBuffer = int(buffer)
	default:
		manager.Logger.Warn("SubscriberMessageBuffer not set, defaulting to 16")
		mq.cs.subscriberMessageBuffer = 100000
	}

	go func() {
		s.Serve(l)
	}()

	return nil
}

func (mq *WebsocketClient) Publish(ctx context.Context, packet *CatwayPayload, channelName string) error {
	if len(packet.Metadata.Shard) < 2 {
		mq.cs.publishGlobal(
			packet,
		)
	} else {
		mq.cs.publish(
			packet.Metadata.Shard,
			packet,
		)
	}

	return nil
}

func (mq *WebsocketClient) IsClosed() bool {
	return mq.cs == nil
}

func (mq *WebsocketClient) CloseShard(shardID int32) {
	// Send RESUME for single shard
	for _, shardSubs := range mq.cs.subscribers {
		for _, s := range shardSubs {
			if s.shard[0] == shardID {
				mq.cs.invalidSession(s, "Shard closed", true)
				mq.cs.deleteSubscriber(s)
			}
		}
	}
}

func (mq *WebsocketClient) Close() {
	// Send RESUME to all shards
	for _, shardSubs := range mq.cs.subscribers {
		for _, s := range shardSubs {
			mq.cs.invalidSession(s, "Connection closed", true)
			s.cancelFunc()
			mq.cs.deleteSubscriber(s)
		}
	}

	mq.cs = nil
}

func randomHex(length int) (result string) {
	buf := make([]byte, length)
	rand.Read(buf)

	return hex.EncodeToString(buf)
}
