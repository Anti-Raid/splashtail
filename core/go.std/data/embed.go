package data

import (
	"embed"
)

//go:embed *
var Embedded embed.FS

//go:embed current-env
var CurrentEnv string
