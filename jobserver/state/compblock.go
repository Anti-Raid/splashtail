// Block certain unsafe configurations by force-importing them
package state

import (
	// Unsafe to import state as used in both webserver and jobserver
	_ "github.com/anti-raid/splashtail/splashcore/animusmagic"
	_ "github.com/anti-raid/splashtail/splashcore/config"
	_ "github.com/anti-raid/splashtail/splashcore/objectstorage"
	_ "github.com/anti-raid/splashtail/splashcore/structparser/db"
	_ "github.com/anti-raid/splashtail/splashcore/types"
	_ "github.com/anti-raid/splashtail/splashcore/utils"
	_ "github.com/anti-raid/splashtail/splashcore/utils/mewext"
	_ "github.com/anti-raid/splashtail/tasks"
)
