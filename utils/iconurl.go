package utils

import "strings"

// https://github.com/bwmarrin/discordgo/blob/master/util.go#L111
func IconURL(iconHash, staticIconURL, animatedIconURL, size string) string {
	var URL string
	if iconHash == "" {
		return ""
	} else if strings.HasPrefix(iconHash, "a_") {
		URL = animatedIconURL
	} else {
		URL = staticIconURL
	}

	if size != "" {
		return URL + "?size=" + size
	}
	return URL
}
