package main

import (
	"errors"
	"regexp"
)

// From https://github.com/diamondburned/arikawa/blob/v3.3.5/api/webhook/webhook.go#L29
var webhookURLRe = regexp.MustCompile(`https://discord(?:app)?.com/api/webhooks/(\d+)/(.+)`)

// ParseURL parses the given Discord webhook URL.
func ParseURL(webhookURL string) (id string, token string, err error) {
	matches := webhookURLRe.FindStringSubmatch(webhookURL)
	if matches == nil {
		return "", "", errors.New("invalid webhook URL")
	}

	return matches[1], matches[2], nil
}
