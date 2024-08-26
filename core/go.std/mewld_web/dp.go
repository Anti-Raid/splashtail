package mewld_web

import (
	"crypto/hmac"
	"crypto/sha512"
	"encoding/hex"
	"net/http"
	"slices"
	"strconv"
	"time"
)

var dpSecret string

func SetState(secret string) {
	dpSecret = secret
}

// Ported from https://github.com/InfinityBotList/sysmanage-web/blob/main/plugins/authdp/mw.go
func DpAuthMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Header.Get("X-DP-Host") == "" {
			w.WriteHeader(http.StatusUnauthorized)
			_, _ = w.Write([]byte("Unauthorized. X-DP-Host header not found. Not running under deployproxy?"))
			return
		}

		if r.Header.Get("X-DP-UserID") == "" {
			// User is not authenticated
			w.WriteHeader(http.StatusUnauthorized)
			_, _ = w.Write([]byte("Unauthorized. Not running under deployproxy?"))
			return
		}

		// User is allowed, set constants.UserIdHeader to user id for other plugins to use it
		r.Header.Set("X-User-ID", r.Header.Get("X-DP-UserID"))

		// Check if user is allowed
		if len(globalConfig.AllowedIDS) > 0 && !slices.Contains(globalConfig.AllowedIDS, r.Header.Get("X-DP-UserID")) {
			w.WriteHeader(http.StatusUnauthorized)
			_, _ = w.Write([]byte("Unauthorized. User not allowed to access this site."))
			return
		}

		// User is possibly allowed
		if r.Header.Get("X-DP-Signature") == "" {
			w.WriteHeader(http.StatusUnauthorized)
			_, _ = w.Write([]byte("Unauthorized. X-DP-Signature header not found."))
			return
		}

		// Check for X-DP-Timestamp
		if r.Header.Get("X-DP-Timestamp") == "" {
			w.WriteHeader(http.StatusUnauthorized)
			_, _ = w.Write([]byte("Unauthorized. X-DP-Timestamp header not found."))
			return
		}

		ts := r.Header.Get("X-DP-Timestamp")

		// Validate DP-Secret next
		if dpSecret != "" {
			h := hmac.New(sha512.New, []byte(dpSecret))
			h.Write([]byte(ts))
			h.Write([]byte(r.Header.Get("X-DP-UserID")))
			hexed := hex.EncodeToString(h.Sum(nil))

			if r.Header.Get("X-DP-Signature") != hexed {
				w.WriteHeader(http.StatusUnauthorized)
				_, _ = w.Write([]byte("Unauthorized. Signature from deployproxy mismatch"))
				return
			}
		}

		// Check if timestamp is valid
		timestamp, err := strconv.ParseInt(ts, 10, 64)

		if err != nil {
			w.WriteHeader(http.StatusUnauthorized)
			_, _ = w.Write([]byte("Unauthorized. X-DP-Timestamp is not a valid integer."))
			return
		}

		if time.Now().Unix()-timestamp > 10 {
			w.WriteHeader(http.StatusUnauthorized)
			_, _ = w.Write([]byte("Unauthorized. X-DP-Timestamp is too old."))
			return
		}

		next.ServeHTTP(w, r)
	})
}
