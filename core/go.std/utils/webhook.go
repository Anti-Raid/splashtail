package utils

import (
	"errors"
	"regexp"
)

type ParsedWebhookUrl struct {
	ID    string
	Token string
}

// From https://github.com/diamondburned/arikawa/blob/v3.3.5/api/webhook/webhook.go#L29
var webhookURLRe = regexp.MustCompile(`https://discord(?:app)?.com/api/webhooks/(\d+)/(.+)`)

// ParseURL parses the given Discord webhook URL.
func ParseWebhookURL(webhookURL string) (pwUrl *ParsedWebhookUrl, err error) {
	matches := webhookURLRe.FindStringSubmatch(webhookURL)
	if matches == nil {
		return nil, errors.New("invalid webhook URL")
	}

	return &ParsedWebhookUrl{
		ID:    matches[1],
		Token: matches[2],
	}, nil
}
