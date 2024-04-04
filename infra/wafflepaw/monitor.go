package main

import (
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"time"

	"github.com/anti-raid/splashtail/cmd/wafflepaw/bgtasks"
	"github.com/anti-raid/splashtail/splashcore/animusmagic"
	"github.com/anti-raid/splashtail/splashcore/config"
	mconfig "github.com/cheesycod/mewld/config"
	mproc "github.com/cheesycod/mewld/proc"
	"go.uber.org/zap"
	"gopkg.in/yaml.v3"
)

var (
	getGatewayBot        *discordGetGatewayBot
	botMldConfig         *mconfig.CoreConfig
	botClusterMap        []mproc.ClusterMap
	jobserverMldConfig   *mconfig.CoreConfig
	jobserverClusterMap  []mproc.ClusterMap
	botAmProbeTask       *AMProbeTask
	jobserverAmProbeTask *AMProbeTask
)

func StartMonitors() (err error) {
	Logger.Info("Starting animus magic client for monitoring main bot")

	// First check number of shards recommended
	getGatewayBot, err = GetDiscordGetGatewayBot()

	if err != nil {
		return fmt.Errorf("failed to get gateway bot: %w", err)
	}

	Logger.Debug("Got gateway bot", zap.Any("gatewayBot", getGatewayBot), zap.Uint64("shards", getGatewayBot.Shards))

	// Next load mewld related yaml files
	wmldF, err := os.ReadFile("data/mewld/botv2-" + config.CurrentEnv + ".yaml")

	if err != nil {
		panic(err)
	}

	var wmldConfig mconfig.CoreConfig

	err = yaml.Unmarshal(wmldF, &wmldConfig)

	if err != nil {
		panic(err)
	}

	// Load mewld bot
	mldF, err := os.ReadFile("data/mewld/jobs-" + config.CurrentEnv + ".yaml")

	if err != nil {
		panic(err)
	}

	var mldConfig mconfig.CoreConfig

	err = yaml.Unmarshal(mldF, &mldConfig)

	if err != nil {
		panic(err)
	}

	Logger.Info("Setting up mewld")

	if mldConfig.Redis == "" {
		mldConfig.Redis = Config.Meta.RedisURL.Parse()
	}

	if mldConfig.Redis != Config.Meta.RedisURL.Parse() {
		Logger.Warn("Redis URL in mewld.yaml does not match the one in config.yaml")
	}

	for _, clusterName := range wmldConfig.Names {
		var i uint64
		for i < wmldConfig.PerCluster {
			mldConfig.Names = append(mldConfig.Names, clusterName+"@"+strconv.FormatUint(i, 10))
			i++
		}
	}

	botMldConfig = &wmldConfig
	jobserverMldConfig = &mldConfig

	// Get clusters from this
	botClusterMap = mproc.GetClusterList(botMldConfig.Names, getGatewayBot.Shards, botMldConfig.PerCluster)
	jobserverClusterMap = mproc.GetClusterList(jobserverMldConfig.Names, getGatewayBot.Shards, jobserverMldConfig.PerCluster)

	AnimusMagicClient = animusmagic.New(Config.Meta.AnimusMagicChannel.Parse(), animusmagic.AnimusTargetInfra, 0)

	go AnimusMagicClient.Listen(Context, Rueidis, Logger)

	botAmProbeTask = &AMProbeTask{
		AnimusMagicClient: AnimusMagicClient,
		Target:            animusmagic.AnimusTargetBot,
		ClusterMap:        botClusterMap,
		MewldChannel:      botMldConfig.RedisChannel,
		SystemdService:    "splashtail-" + config.CurrentEnv + "-webserver",
	}

	jobserverAmProbeTask = &AMProbeTask{
		AnimusMagicClient: AnimusMagicClient,
		Target:            animusmagic.AnimusTargetBot,
		ClusterMap:        botClusterMap,
		MewldChannel:      botMldConfig.RedisChannel,
		SystemdService:    "splashtail-" + config.CurrentEnv + "-webserver",
	}

	bgtasks.BgTaskRegistry = append(bgtasks.BgTaskRegistry, botAmProbeTask)
	bgtasks.BgTaskRegistry = append(bgtasks.BgTaskRegistry, jobserverAmProbeTask)

	return nil
}

type AMProbeTask struct {
	AnimusMagicClient *animusmagic.AnimusMagicClient
	Target            animusmagic.AnimusTarget
	ClusterMap        []mproc.ClusterMap
	MewldChannel      string
	SystemdService    string
}

func (p *AMProbeTask) Enabled() bool {
	return true
}

func (p *AMProbeTask) Duration() time.Duration {
	return 5 * time.Minute
}

func (p *AMProbeTask) Name() string {
	return "AMProbe"
}

func (p *AMProbeTask) Description() string {
	return "Probes for animus target " + p.Target.String()
}

func (p *AMProbeTask) Run() error {
	commandId := animusmagic.NewCommandId()
	payload, err := p.AnimusMagicClient.CreatePayload(
		p.AnimusMagicClient.From,
		p.Target,
		p.AnimusMagicClient.ClusterID,
		animusmagic.WildcardClusterID,
		animusmagic.OpProbe,
		commandId,
		[]byte{},
	)

	if err != nil {
		return fmt.Errorf("error creating payload: %s", err)
	}

	// Create a channel to receive the response
	notify := p.AnimusMagicClient.CreateNotifier(commandId, 0)

	// Publish the payload
	err = p.AnimusMagicClient.Publish(Context, Rueidis, payload)

	if err != nil {
		// Remove the notifier
		p.AnimusMagicClient.CloseNotifier(commandId)
		return fmt.Errorf("error publishing payload: %s", err)
	}

	// Wait for the response
	waitForResponse := func() ([]uint16, error) {
		var clusterIds []uint16

		ticker := time.NewTicker(time.Second * 10)
		startTime := time.Now()
		for {
			select {
			case <-Context.Done():
				return nil, fmt.Errorf("context cancelled")
			case <-ticker.C:
				return clusterIds, nil
			case response := <-notify:
				since := time.Since(startTime)

				if since > time.Second*5 {
					Logger.Warn("AMProbe response took too longer than usual", zap.Duration("duration", since))
				}

				clusterIds = append(clusterIds, response.Meta.ClusterIDFrom)
			}
		}
	}

	clusterIds, err := waitForResponse()

	if err != nil {
		Logger.Error("Error waiting for response", zap.Error(err))
		return p.tryRestartService()
	}

	Logger.Debug("AMProbe response", zap.Any("clusterIds", clusterIds))

	return nil
}

// tryRestartService tries to restart the entire service using systemd
func (p *AMProbeTask) tryRestartService() error {
	Logger.Debug("Restarting service", zap.String("service", p.SystemdService))
	cmd := exec.Command("systemctl", "restart", p.SystemdService)

	err := cmd.Run()

	if err != nil {
		return fmt.Errorf("error restarting service: %s", err)
	}

	return nil
}
