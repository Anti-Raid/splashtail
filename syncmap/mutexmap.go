package syncmap

import "sync"

type MutexedMap[K comparable, V any] struct {
	sync.RWMutex
	Map map[K]V
}

func (m *MutexedMap[K, V]) Load(key K) (V, bool) {
	m.RLock()
	defer m.RUnlock()
	val, ok := m.Map[key]
	return val, ok
}

func (m *MutexedMap[K, V]) Store(key K, val V) {
	m.Lock()
	defer m.Unlock()
	m.Map[key] = val
}

func (m *MutexedMap[K, V]) Delete(key K) {
	m.Lock()
	defer m.Unlock()
	delete(m.Map, key)
}

func (m *MutexedMap[K, V]) Length() int {
	m.RLock()
	defer m.RUnlock()
	return len(m.Map)
}
