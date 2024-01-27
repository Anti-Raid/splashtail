// Block certain unsafe configurations by force-importing them
package state

import (
	// Unsafe to import state as used in both webserver and jobserver
	_ "github.com/anti-raid/splashtail/animusmagic"
	_ "github.com/anti-raid/splashtail/config"
	_ "github.com/anti-raid/splashtail/db"
	_ "github.com/anti-raid/splashtail/objectstorage"
	_ "github.com/anti-raid/splashtail/tasks"
	_ "github.com/anti-raid/splashtail/types"
	_ "github.com/anti-raid/splashtail/utils"
	_ "github.com/anti-raid/splashtail/utils/mewext"
)
