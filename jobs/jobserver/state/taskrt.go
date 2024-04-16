package state

import (
	"fmt"
	"net/http"
	"time"
)

var taskRtDefaultExp = 5 * time.Minute

type TaskRT struct {
	next http.RoundTripper
}

func (t TaskRT) RoundTrip(req *http.Request) (resp *http.Response, err error) {
	// Create presigned url
	expiry := req.URL.Query().Get("exp")

	var expiryDuration time.Duration

	if expiry != "" {
		expiryDuration, err = time.ParseDuration(expiry)

		if err != nil {
			return nil, err
		}
	} else {
		expiryDuration = taskRtDefaultExp
	}

	fmt.Println(req.URL.Path)

	url, err := ObjectStorage.GetUrl(req.Context(), req.URL.Path, "", expiryDuration)

	if err != nil {
		return nil, err
	}

	req.URL = url
	req.Host = url.Host

	return t.next.RoundTrip(req)
}
