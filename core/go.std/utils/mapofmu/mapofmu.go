// Package mapofmu provides locking per-key.
// For example, you can acquire a lock for a specific user ID and all other requests for that user ID
// will block until that entry is unlocked (effectively your work load will be run serially per-user ID),
// and yet have work for separate user IDs happen concurrently.
//
// https://stackoverflow.com/questions/40931373/how-to-gc-a-map-of-mutexes-in-go
package mapofmu

import (
	"fmt"
	"sync"
)

// M wraps a map of mutexes.  Each key locks separately.
type M[K comparable] struct {
	ml sync.Mutex       // lock for entry map
	ma map[K]*mentry[K] // entry map
}

type mentry[K comparable] struct {
	m   *M[K]      // point back to M, so we can synchronize removing this mentry when cnt==0
	el  sync.Mutex // entry-specific lock
	cnt int        // reference count
	key K          // key in ma
}

// Unlocker provides an Unlock method to release the lock.
type Unlocker interface {
	Unlock()
}

// New returns an initalized M.
func New[K comparable]() *M[K] {
	return &M[K]{ma: make(map[K]*mentry[K])}
}

// Lock acquires a lock corresponding to this key.
// This method will never return nil and Unlock() must be called
// to release the lock when done.
func (m *M[K]) Lock(key K) Unlocker {

	// read or create entry for this key atomically
	m.ml.Lock()
	e, ok := m.ma[key]
	if !ok {
		e = &mentry[K]{m: m, key: key}
		m.ma[key] = e
	}
	e.cnt++ // ref count
	m.ml.Unlock()

	// acquire lock, will block here until e.cnt==1
	e.el.Lock()

	return e
}

// IsLocked returns true if the key is locked.
func (m *M[K]) IsLocked(key K) bool {
	m.ml.Lock()
	_, ok := m.ma[key]
	m.ml.Unlock()
	return ok
}

// Unlock releases the lock for this entry.
func (me *mentry[K]) Unlock() {

	m := me.m

	// decrement and if needed remove entry atomically
	m.ml.Lock()
	e, ok := m.ma[me.key]
	if !ok { // entry must exist
		m.ml.Unlock()
		panic(fmt.Errorf("Unlock requested for key=%v but no entry found", me.key))
	}
	e.cnt--        // ref count
	if e.cnt < 1 { // if it hits zero then we own it and remove from map
		delete(m.ma, me.key)
	}
	m.ml.Unlock()

	// now that map stuff is handled, we unlock and let
	// anything else waiting on this key through
	e.el.Unlock()

}
