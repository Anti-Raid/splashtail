// Concurrent safe value wrapper
package rwvalue

import "sync"

type RWValue[T any] struct {
	ml sync.RWMutex
	v  *T
}

func New[T any](v *T) *RWValue[T] {
	return &RWValue[T]{v: v}
}

func (v *RWValue[T]) Get() *T {
	v.ml.RLock()
	defer v.ml.RUnlock()
	return v.v
}

func (v *RWValue[T]) Set(val *T) {
	v.ml.Lock()
	v.v = val
	v.ml.Unlock()
}

func (v *RWValue[T]) Clear() {
	v.ml.Lock()
	v.v = nil
	v.ml.Unlock()
}
