package rwmap

import "sync"

type RWMap[K comparable, V any] struct {
	ml sync.RWMutex

	m map[K]V
}

func New[K comparable, V any]() *RWMap[K, V] {
	return &RWMap[K, V]{m: make(map[K]V)}
}

// Has returns true if the given key exists in the map.
func (m *RWMap[K, V]) Has(key K) bool {
	m.ml.RLock()
	_, ok := m.m[key]
	m.ml.RUnlock()
	return ok
}

// Get returns the value associated with the given key, and a bool indicating
// whether the key was found.
func (m *RWMap[K, V]) Get(key K) (V, bool) {
	m.ml.RLock()
	v, ok := m.m[key]
	m.ml.RUnlock()
	return v, ok
}

// Set sets the value associated with the given key
// locking until the map is available for writing.
func (m *RWMap[K, V]) Set(key K, value V) {
	m.ml.Lock()
	m.m[key] = value
	m.ml.Unlock()
}

// Delete deletes the value associated with the given key
// locking until the map is available for writing.
func (m *RWMap[K, V]) Delete(key K) {
	m.ml.Lock()
	delete(m.m, key)
	m.ml.Unlock()
}

// Len returns the number of items in the map.
func (m *RWMap[K, V]) Len() int {
	m.ml.RLock()
	l := len(m.m)
	m.ml.RUnlock()
	return l
}

// Keys returns a slice of all keys in the map.
func (m *RWMap[K, V]) Keys() []K {
	m.ml.RLock()
	keys := make([]K, 0, len(m.m))
	for k := range m.m {
		keys = append(keys, k)
	}
	m.ml.RUnlock()
	return keys
}

// Values returns a slice of all values in the map.
func (m *RWMap[K, V]) Values() []V {
	m.ml.RLock()
	values := make([]V, 0, len(m.m))
	for _, v := range m.m {
		values = append(values, v)
	}
	m.ml.RUnlock()
	return values
}

// Clear removes all items from the map.
func (m *RWMap[K, V]) Clear() {
	m.ml.Lock()
	m.m = make(map[K]V)
	m.ml.Unlock()
}

// Copy returns a shallow copy of the map.
func (m *RWMap[K, V]) Copy() *RWMap[K, V] {
	m.ml.RLock()
	m2 := New[K, V]()
	for k, v := range m.m {
		m2.m[k] = v
	}
	m.ml.RUnlock()
	return m2
}
