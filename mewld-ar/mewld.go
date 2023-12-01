package mewld

import (
	"mewld/config"
	"mewld/coreutils"
	"mewld/proc"
	"mewld/redis"
	"mewld/utils"
	"mewld/web"
	"os"
	"os/signal"
	"strconv"
	"syscall"

	_ "embed"

	log "github.com/sirupsen/logrus"
	"gopkg.in/yaml.v3"
)

// ANTIRAID-SPECIFIC: Expose InstanceList as a global
var InstanceList *proc.InstanceList

//go:embed mewld.yaml
var configBytes []byte

func init() {
	lvl, ok := os.LookupEnv("LOG_LEVEL")
	// LOG_LEVEL not set, let's default to info
	if !ok {
		lvl = "info"
	}
	// parse string, this is built-in feature of logrus
	ll, err := log.ParseLevel(lvl)
	if err != nil {
		ll = log.InfoLevel
	}
	// set global log level
	log.SetLevel(ll)
}

// ANTIRAID-SPECIFIC: Rename main to Load to allow embedding
//
// Also allow manualOauth and manualToken to be passed instead of relying on env
func Load(manualOauth *config.Oauth, manualToken *string) {
	// Load the config file
	var config config.CoreConfig

	err := yaml.Unmarshal(configBytes, &config)

	if err != nil {
		log.Fatal("Check config file again: ", err)
	}

	// ANTIRAID-SPECIFIC: Allow manually passing oauth2 and bot credentials
	if manualOauth != nil {
		config.Oauth = *manualOauth
	}

	if manualToken != nil {
		config.Token = *manualToken
	}

	var dir string
	if config.OverrideDir != "" {
		dir = config.OverrideDir
	} else {
		dirname, err := os.Getwd() // ANTIRAID-SPECIFIC: Use Getwd instead of UserHomeDir
		if err != nil {
			log.Fatal("Could not find Getwd directory: ", err)
		}

		dir = dirname + "/" + config.Dir
	}

	err = os.Chdir(dir)

	if err != nil {
		log.Fatal("Could not change into directory: ", err)
	}

	// ANTIRAID-SPECIFIC: Dont load .env files at all

	shardCount := web.GetShardCount(config)

	log.Println("Recommended shard count:", shardCount.Shards)

	if os.Getenv("SHARD_COUNT") != "" {
		shardCount.Shards = coreutils.ParseUint64(os.Getenv("SHARD_COUNT"))
	}

	var perCluster uint64 = config.PerCluster

	if os.Getenv("PER_CLUSTER") != "" {
		perCluster = coreutils.ParseUint64(os.Getenv("PER_CLUSTER"))
	}

	log.Println("Cluster names:", config.Names)

	clusterMap := utils.GetClusterList(config.Names, shardCount.Shards, perCluster)

	il := proc.InstanceList{
		Config:     config,
		Dir:        dir,
		Map:        clusterMap,
		Instances:  []*proc.Instance{},
		ShardCount: shardCount.Shards,
	}

	il.Init()

	InstanceList = &il

	for _, cMap := range clusterMap {
		log.Info("Cluster ", cMap.Name, "("+strconv.Itoa(cMap.ID)+"): ", coreutils.ToPyListUInt64(cMap.Shards))
		il.Instances = append(il.Instances, &proc.Instance{
			SessionID: coreutils.RandomString(16),
			ClusterID: cMap.ID,
			Shards:    cMap.Shards,
		})
	}

	// Start the redis handler
	redish := redis.CreateHandler(config)
	go redish.Start(&il)

	go web.StartWebserver(web.WebData{
		RedisHandler: &redish,
		InstanceList: &il,
	})

	// Wait here until we get a signal
	sigs := make(chan os.Signal, 1)

	signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)

	// We now start the first cluster, this cluster will then alert us over redis when to start cluster 2 (todo: timeout?)
	il.Start(il.Instances[0])

	sig := <-sigs

	log.Info("Received signal: ", sig)

	il.KillAll()

	// Exit
	os.Exit(0)
}
