// Core utilities to break import cycles
package coreutils

import (
	"fmt"
	"log"
	"math/rand"
	"strconv"
	"time"
	"unsafe"
)

// Creates a python compatible list
func ToPyListUInt64(l []uint64) string {
	var s string = "["
	for i, v := range l {
		s += fmt.Sprint(v)
		if i != len(l)-1 {
			s += ", "
		}
	}
	return s + "]"
}

func ParseUint64(s string) uint64 {
	i, err := strconv.ParseUint(s, 10, 64)

	if err != nil {
		log.Fatal(err)
	}

	return i
}

func UInt64ToString(i uint64) string {
	return strconv.FormatUint(i, 10)
}

const letterBytes = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
const (
	letterIdxBits = 6                    // 6 bits to represent a letter index
	letterIdxMask = 1<<letterIdxBits - 1 // All 1-bits, as many as letterIdxBits
	letterIdxMax  = 63 / letterIdxBits   // # of letter indices fitting in 63 bits
)

var src = rand.NewSource(time.Now().UnixNano())

func RandomString(n int) string {
	b := make([]byte, n)
	// A src.Int63() generates 63 random bits, enough for letterIdxMax characters!
	for i, cache, remain := n-1, src.Int63(), letterIdxMax; i >= 0; {
		if remain == 0 {
			cache, remain = src.Int63(), letterIdxMax
		}
		if idx := int(cache & letterIdxMask); idx < len(letterBytes) {
			b[i] = letterBytes[idx]
			i--
		}
		cache >>= letterIdxBits
		remain--
	}

	return *(*string)(unsafe.Pointer(&b))
}
