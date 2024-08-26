package redishotcache

import (
	"context"
	"encoding/json"
	"errors"
	"time"

	"github.com/infinitybotlist/eureka/hotcache"
	"github.com/redis/rueidis"
)

type RuedisHotCache[T any] struct {
	Redis    rueidis.Client
	Prefix   string
	For      string
	Disabled bool
}

func (r RuedisHotCache[T]) Get(ctx context.Context, key string) (*T, error) {
	if r.Disabled {
		return nil, hotcache.ErrHotCacheDataNotFound
	}

	bytes, err := r.Redis.Do(ctx, r.Redis.B().Get().Key(r.Prefix+key).Build()).AsBytes()

	if errors.Is(err, rueidis.Nil) {
		return nil, hotcache.ErrHotCacheDataNotFound
	}

	if err != nil {
		return nil, err
	}

	var val T

	err = json.Unmarshal(bytes, &val)

	if err != nil {
		return nil, err
	}

	return &val, nil
}

func (r RuedisHotCache[T]) Delete(ctx context.Context, key string) error {
	if r.Disabled {
		return nil
	}

	return r.Redis.Do(ctx, r.Redis.B().Del().Key(r.Prefix+key).Build()).Error()
}

func (r RuedisHotCache[T]) Set(ctx context.Context, key string, value *T, expiry time.Duration) error {
	if r.Disabled {
		return nil
	}

	bytes, err := json.Marshal(value)

	if err != nil {
		return err
	}

	return r.Redis.Do(ctx, r.Redis.B().Set().Key(r.Prefix+key).Value(string(bytes)).Ex(expiry).Build()).Error()
}

func (r RuedisHotCache[T]) Increment(ctx context.Context, key string, value int64) error {
	if r.Disabled {
		return nil
	}

	return r.Redis.Do(ctx, r.Redis.B().Incrby().Key(r.Prefix+key).Increment(value).Build()).Error()
}

func (r RuedisHotCache[T]) IncrementOne(ctx context.Context, key string) error {
	if r.Disabled {
		return nil
	}

	return r.Redis.Do(ctx, r.Redis.B().Incr().Key(r.Prefix+key).Build()).Error()
}

func (r RuedisHotCache[T]) Exists(ctx context.Context, key string) (bool, error) {
	if r.Disabled {
		return false, nil
	}

	b, err := r.Redis.Do(ctx, r.Redis.B().Exists().Key(r.Prefix+key).Build()).AsInt64()

	if err != nil {
		return false, err
	}

	return b > 0, nil
}

func (r RuedisHotCache[T]) Expiry(ctx context.Context, key string) (time.Duration, error) {
	if r.Disabled {
		return 0, nil
	}

	b, err := r.Redis.Do(ctx, r.Redis.B().Ttl().Key(r.Prefix+key).Build()).AsInt64()

	if err != nil {
		return 0, err
	}

	return time.Duration(b) * time.Second, nil
}
