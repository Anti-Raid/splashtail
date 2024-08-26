package timex

import (
	"encoding/json"
	"errors"
	"time"
)

// Duration is a wrapper around time.Duration that allows for JSON marshalling and unmarshalling
type Duration time.Duration

var Nanosecond = Duration(time.Nanosecond)
var Microsecond = Duration(time.Microsecond)
var Millisecond = Duration(time.Millisecond)
var Second = Duration(time.Second)
var Minute = Duration(time.Minute)
var Hour = Duration(time.Hour)
var Day = Duration(time.Hour * 24)
var Week = Duration(time.Hour * 24 * 7)
var Month = Duration(time.Hour * 24 * 30)

func (d Duration) MarshalJSON() ([]byte, error) {
	return json.Marshal(time.Duration(d).String())
}

func (d *Duration) UnmarshalJSON(b []byte) error {
	var v interface{}
	if err := json.Unmarshal(b, &v); err != nil {
		return err
	}
	switch value := v.(type) {
	case float64:
		*d = Duration(time.Duration(value))
		return nil
	case string:
		tmp, err := time.ParseDuration(value)
		if err != nil {
			return err
		}
		*d = Duration(tmp)
		return nil
	default:
		return errors.New("invalid duration")
	}
}
