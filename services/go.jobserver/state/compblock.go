// Block certain unsafe configurations by force-importing them
package state

import (
	// Unsafe to import state as used in both webserver and jobserver
	_ "github.com/anti-raid/splashtail/core/go.jobs"
	_ "github.com/anti-raid/splashtail/core/go.std/animusmagic"
	_ "github.com/anti-raid/splashtail/core/go.std/config"
	_ "github.com/anti-raid/splashtail/core/go.std/objectstorage"
	_ "github.com/anti-raid/splashtail/core/go.std/structparser/db"
	_ "github.com/anti-raid/splashtail/core/go.std/utils"
	_ "github.com/anti-raid/splashtail/core/go.std/utils/mewext"
)
