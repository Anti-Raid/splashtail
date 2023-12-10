package utils

import (
	"time"
)

func Timed(f func()) time.Duration {
	t2 := time.Now()

	f()

	t1 := time.Now()

	return t1.Sub(t2)
}
