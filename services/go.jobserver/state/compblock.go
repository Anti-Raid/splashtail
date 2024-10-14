// Block certain unsafe configurations by force-importing them
package state

import (
	// Unsafe to import state as used in both webserver and jobserver
	_ "go.jobs"
	_ "go.std/config"
	_ "go.std/objectstorage"
	_ "go.std/structparser/db"
	_ "go.std/utils"
)
