package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"time"

	"github.com/anti-raid/splashtail/cmd/wafflepaw/bgtasks"
	"github.com/anti-raid/splashtail/splashcore/animusmagic"
	"github.com/anti-raid/splashtail/splashcore/config"
	"github.com/bwmarrin/discordgo"
	mconfig "github.com/cheesycod/mewld/config"
	mproc "github.com/cheesycod/mewld/proc"
	mredis "github.com/cheesycod/mewld/redis"
	"go.uber.org/zap"
	"gopkg.in/yaml.v3"
)

var (
	getGatewayBot        *discordgo.GatewayBotResponse
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
	getGatewayBot, err = Discord.GatewayBot(discordgo.WithContext(Context))

	if err != nil {
		return fmt.Errorf("failed to get gateway bot: %w", err)
	}

	Logger.Debug("Got gateway bot", zap.Any("gatewayBot", getGatewayBot), zap.Int("shards", getGatewayBot.Shards))

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
	botClusterMap = mproc.GetClusterList(botMldConfig.Names, uint64(getGatewayBot.Shards), botMldConfig.PerCluster)
	jobserverClusterMap = mproc.GetClusterList(jobserverMldConfig.Names, uint64(getGatewayBot.Shards), jobserverMldConfig.PerCluster)

	Logger.Info("Cluster maps generated", zap.Any("botClusterMap", botClusterMap), zap.Any("jobserverClusterMap", jobserverClusterMap))

	AnimusMagicClient = animusmagic.New(Config.Meta.AnimusMagicChannel.Parse(), animusmagic.AnimusTargetInfra, 0)

	go AnimusMagicClient.Listen(Context, Rueidis, Logger)

	botAmProbeTask = &AMProbeTask{
		AnimusMagicClient:       AnimusMagicClient,
		Target:                  animusmagic.AnimusTargetBot,
		ClusterMap:              botClusterMap,
		MewldChannel:            botMldConfig.RedisChannel,
		SystemdService:          "splashtail-" + config.CurrentEnv + "-webserver",
		NoHandleInactiveSystemd: true,
		RestartAfterFailed:      3,
	}

	jobserverAmProbeTask = &AMProbeTask{
		AnimusMagicClient:       AnimusMagicClient,
		Target:                  animusmagic.AnimusTargetJobserver,
		ClusterMap:              jobserverClusterMap,
		MewldChannel:            jobserverMldConfig.RedisChannel,
		SystemdService:          "splashtail-" + config.CurrentEnv + "-jobs",
		DelayStart:              10 * time.Second,
		NoHandleInactiveSystemd: true,
	}

	bgtasks.BgTaskRegistry = append(bgtasks.BgTaskRegistry, botAmProbeTask)
	bgtasks.BgTaskRegistry = append(bgtasks.BgTaskRegistry, jobserverAmProbeTask)

	return nil
}

// Internal task state. TaskMutex in bgtasks guarantees that only one task is running at a time.
type probeTaskState struct {
	LastProbeTime                       time.Time
	LastSuccessfulProbeTime             time.Time
	AttemptedMewldClusterRollingRestart bool
	AttemptedSystemdRestart             bool
	FailedCount                         int
}

type AMProbeTask struct {
	AnimusMagicClient       *animusmagic.AnimusMagicClient
	Target                  animusmagic.AnimusTarget
	ClusterMap              []mproc.ClusterMap
	MewldChannel            string
	SystemdService          string
	DelayStart              time.Duration
	NoHandleInactiveSystemd bool
	RestartAfterFailed      int // After how many failed checks should we restart the service

	// Internal state
	state probeTaskState
}

func (p *AMProbeTask) Enabled() bool {
	return true
}

func (p *AMProbeTask) Duration() time.Duration {
	return 30*time.Second + p.DelayStart
}

func (p *AMProbeTask) Name() string {
	return "AMProbe"
}

func (p *AMProbeTask) Description() string {
	return "Probes for animus target " + p.Target.String()
}

func (p *AMProbeTask) Run() error {
	p.state.LastProbeTime = time.Now()

	serviceStatus, err := p.getServiceStatus()

	if err != nil {
		return fmt.Errorf("error getting service status: %s", err)
	}

	Logger.Debug("Service status", zap.String("service", p.SystemdService), zap.String("status", serviceStatus))

	if p.NoHandleInactiveSystemd && serviceStatus == "inactive" {
		Logger.Debug("Service is inactive, skipping AMProbe")
		return nil
	}

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
		return fmt.Errorf("error waiting for response: %s", err)
	}

	// Check that there are no duplicate cluster ids
	clusterIdMap := make(map[uint16]struct{})

	duplicateClusterIds := []uint16{}
	for _, clusterId := range clusterIds {
		if _, ok := clusterIdMap[clusterId]; ok {
			duplicateClusterIds = append(duplicateClusterIds, clusterId)
		}
		clusterIdMap[clusterId] = struct{}{}
	}

	// If we have duplicate cluster ids, try to restart problematic clusters
	if len(duplicateClusterIds) > 0 {
		p.restart(tryRestartOptions{
			ProblematicClusters: duplicateClusterIds,
		})
	}

	// If we have less than half the expected clusters, try to restart all
	if len(clusterIds) < len(p.ClusterMap)/2 {
		p.restart(tryRestartOptions{})
	}

	Logger.Debug("AMProbe response", zap.Any("clusterIds", clusterIds))

	p.state.LastSuccessfulProbeTime = time.Now()
	p.resetAttempts()
	return nil
}

// resetAttempts resets the state of the task related to attempts
func (p *AMProbeTask) resetAttempts() {
	p.state.AttemptedMewldClusterRollingRestart = false
	p.state.AttemptedSystemdRestart = false
	p.state.FailedCount = 0
}

type tryRestartOptions struct {
	ProblematicClusters []uint16 // Any specific problematic clusters
}

// restart tries to restart the service only when failedCount is greater than RestartAfterFailed
// (with some other logic such as sending a webhook). Consumers should use restart over tryRestart hence.
func (p *AMProbeTask) restart(opts tryRestartOptions) error {
	p.state.FailedCount++
	if p.state.FailedCount > p.RestartAfterFailed {
		// Send a webhook to notify
		var webhookContext = discordgo.WebhookParams{
			Content: fmt.Sprintf("@everyone **CRITICAL ALERT** Restarting service %s due to failed probes (>%d failed probes)", p.SystemdService, p.RestartAfterFailed),
		}

		id, token, err := ParseURL(Config.Wafflepaw.StatusWebhook)

		if err == nil {
			_, err = Discord.WebhookExecute(id, token, false, &webhookContext)

			if err != nil {
				Logger.Error("Error sending webhook", zap.Error(err))
			}
		} else {
			Logger.Error("Error parsing webhook URL", zap.Error(err))
		}

		err = p.tryRestart(opts)

		if err != nil {
			return fmt.Errorf("error restarting service: %s", err)
		}

		p.state.FailedCount = 0 // Reset failed count
	}
	return nil
}

// tryRestart tries to first restart via mewld and return.
//
// If, at the next run, we still go through this function, then we try
// the tryRestartServiceSystemd method
func (p *AMProbeTask) tryRestart(opts tryRestartOptions) error {
	Logger.Info("Attempting restart")
	if len(opts.ProblematicClusters) > 0 {
		// Just log for now, restarting individual clusters is not implemented yet
		Logger.Debug("Problematic clusters detected", zap.Any("clusters", opts.ProblematicClusters))
	}

	if !p.state.AttemptedMewldClusterRollingRestart {
		Logger.Debug("Attempting mewld cluster restart")
		err := p.tryRollingRestartMewldCluster()

		if err != nil {
			return fmt.Errorf("error restarting mewld cluster: %s", err)
		}

		p.state.AttemptedMewldClusterRollingRestart = true

		// Give some extra buffer time
		time.Sleep(3 * time.Second)

		return nil
	}

	// If we reach here, we have already tried restarting the mewld cluster
	if !p.state.AttemptedSystemdRestart {
		Logger.Debug("Attempting service restart")
		err := p.tryRestartServiceSystemd()

		if err != nil {
			return fmt.Errorf("error restarting service: %s", err)
		}

		p.state.AttemptedSystemdRestart = true

		// Give some extra buffer time
		time.Sleep(3 * time.Second)
	}

	// Fallback to tryRestartServiceSystemd
	// This is the last resort
	Logger.Error("Failed to restart service via mewld, attempting systemd restart")

	id, token, err := ParseURL(Config.Wafflepaw.StatusWebhook)

	if err == nil {
		_, err = Discord.WebhookExecute(id, token, false, &discordgo.WebhookParams{
			Content: fmt.Sprintf("@everyone **CRITICAL ALERT** Failed to restart service %s via mewld, attempting systemd restart", p.SystemdService),
		})

		if err != nil {
			Logger.Error("Error sending webhook", zap.Error(err))
		}
	}

	err = p.tryRestartServiceSystemd()

	if err != nil {
		return fmt.Errorf("error restarting service: %s", err)
	}

	// Give some extra buffer time
	time.Sleep(5 * time.Second)

	return nil
}

// getServiceStatus returns the systemd status of the service
// e.g. "active", "inactive", "failed"
func (p *AMProbeTask) getServiceStatus() (string, error) {
	cmd := exec.Command("systemctl", "show", p.SystemdService, "--property=ActiveState", "--value")

	out, err := cmd.Output()

	if err != nil {
		return "", fmt.Errorf("error getting service status: %s", err)
	}

	// Remove newline
	output := strings.Trim(string(out), "\n")

	return output, nil
}

// tryRestartMewldCluster tries to restart a specific cluster with an id
//
// TODO: Implement this
/*func (p *AMProbeTask) tryRestartMewldCluster(clusterId uint16) error {
	return nil
}*/

// tryRollingRestartMewldCluster tries to restart the mewld cluster
// using a rolling restart
func (p *AMProbeTask) tryRollingRestartMewldCluster() error {
	rr := mredis.LauncherCmd{
		Scope:     "launcher",
		Action:    "rollingrestart",
		CommandId: animusmagic.NewCommandId(),
	}

	bytes, err := json.Marshal(rr)

	if err != nil {
		return fmt.Errorf("error marshalling rolling restart command: %s", err)
	}

	publishCmd := Rueidis.B().Publish().Channel(p.MewldChannel).Message(string(bytes)).Build()

	err = Rueidis.Do(Context, publishCmd).Error()

	if err != nil {
		return fmt.Errorf("error publishing rolling restart command: %s", err)
	}

	return nil
}

// tryRestartServiceSystemd tries to restart the entire service using systemd
func (p *AMProbeTask) tryRestartServiceSystemd() error {
	Logger.Debug("Restarting service", zap.String("service", p.SystemdService))
	cmd := exec.Command("systemctl", "restart", p.SystemdService)

	err := cmd.Run()

	if err != nil {
		return fmt.Errorf("error restarting service: %s", err)
	}

	return nil
}
