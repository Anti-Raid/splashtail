package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"os/exec"
	"slices"
	"strconv"
	"strings"
	"time"

	"github.com/anti-raid/splashtail/cmd/wafflepaw/bgtasks"
	"github.com/bwmarrin/discordgo"
	mconfig "github.com/cheesycod/mewld/config"
	mproc "github.com/cheesycod/mewld/proc"
	mredis "github.com/cheesycod/mewld/redis"
	"github.com/infinitybotlist/eureka/crypto"
	"go.std/config"
	"go.uber.org/zap"
	"gopkg.in/yaml.v3"
)

var (
	getGatewayBot       *discordgo.GatewayBotResponse
	botMldConfig        *mconfig.CoreConfig
	botClusterMap       []mproc.ClusterMap
	jobserverMldConfig  *mconfig.CoreConfig
	jobserverClusterMap []mproc.ClusterMap
)

func StartMonitors(monitors []ProbeTask) (err error) {
	Logger.Info("Starting wafflepaw probe task for monitoring main bot")

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

	mldConfig.Proxy = Config.Meta.Proxy.Parse()
	wmldConfig.Proxy = Config.Meta.Proxy.Parse()

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

	var clusterMaps = map[string][]mproc.ClusterMap{
		"bot":  botClusterMap,
		"jobs": jobserverClusterMap,
	}

	var mewldChanMap = map[string]string{
		"bot":  botMldConfig.RedisChannel,
		"jobs": jobserverMldConfig.RedisChannel,
	}

	for i := range monitors {
		if len(monitors[i].ClusterMap) == 0 {
			cmap, ok := clusterMaps[monitors[i].Target]

			if !ok {
				monitors[i].ClusterMap = []mproc.ClusterMap{{ID: 0}}
			} else {
				monitors[i].ClusterMap = cmap
			}
		}

		// Set mewld channel if unset and supported, otherwise ignore
		if monitors[i].MewldChannel == "" {
			if ch, ok := mewldChanMap[monitors[i].Target]; ok {
				monitors[i].MewldChannel = ch
			}
		}

		monitors[i].SystemdService = strings.ReplaceAll(monitors[i].SystemdService, "{env}", config.CurrentEnv)

		Logger.Info("Added monitor", zap.Any("monitor", monitors[i]))
		bgtasks.BgTaskRegistry = append(bgtasks.BgTaskRegistry, &monitors[i])

		monitors[i].state.Client.Timeout = 10 * time.Second
	}

	return nil
}

// Internal state info

// Internal task state. TaskMutex in bgtasks guarantees that only one task is running at a time.
type probeTaskState struct {
	Client                              http.Client
	LastProbeTime                       time.Time
	LastSuccessfulProbeTime             time.Time
	AttemptedMewldClusterRollingRestart bool
	AttemptedTargettedKills             []string
	AttemptedKillall                    bool
	AttemptedSystemdRestart             bool
	FailedCount                         int
	BackoffExp                          int // Exponential backoff in restart
}

type ProbeTask struct {
	// Only the below fields are exposed for monitors in monitor.yaml
	ClusterMap              []mproc.ClusterMap `json:"cluster_map" yaml:"cluster_map"`                               // Cluster map
	Target                  string             `json:"target" yaml:"target"`                                         // Target to probe
	MewldChannel            string             `json:"mewld_channel" yaml:"mewld_channel"`                           // Mewld channel to send pings to
	SystemdService          string             `json:"systemd_service" yaml:"systemd_service"`                       // Corresponding Systemd service to restart
	AuxillaryServices       []string           `json:"auxillary_services" yaml:"auxillary_services"`                 // Auxillary services to restart if needed
	DelayStart              time.Duration      `json:"delay_start" yaml:"delay_start"`                               // Delay start
	NoHandleInactiveSystemd bool               `json:"no_handle_inactive_systemd" yaml:"no_handle_inactive_systemd"` // Do not handle inactive systemd services
	RestartAfterFailed      int                `json:"restart_after_failed" yaml:"restart_after_failed"`             // After how many failed checks should we restart the service
	ProcessName             []string           `json:"process_name" yaml:"process_name"`                             // Process names to kill
	BasePort                int                `json:"base_port" yaml:"base_port"`                                   // Base port for the service
	BaseAddr                string             `json:"base_addr" yaml:"base_addr"`                                   // Base address for the service

	// Internal state
	state probeTaskState `yaml:"-"`
}

func (p *ProbeTask) Enabled() bool {
	return true
}

func (p *ProbeTask) Duration() time.Duration {
	return 10*time.Second + p.DelayStart
}

func (p *ProbeTask) Name() string {
	return "AMProbe"
}

func (p *ProbeTask) Description() string {
	return "Probes for target " + p.Target
}

func (p *ProbeTask) Run() error {
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

	if serviceStatus == "failed" {
		Logger.Error("Service is in failed state, restarting")
		return p.tryRestartServiceSystemd()
	}

	var resps []int // A collection of status codes

	for _, cluster := range p.ClusterMap {
		// Make GET request to check if the cluster is alive
		resp, err := p.state.Client.Get(fmt.Sprintf("%s:%d", p.BaseAddr, p.BasePort+cluster.ID))

		if err != nil {
			Logger.Error("Error making request", zap.Error(err))
			resps = append(resps, 500)
			continue
		}

		resps = append(resps, resp.StatusCode)
	}

	// Get the count of 200s
	var count200 int

	for _, resp := range resps {
		if resp == 200 {
			count200++
		}
	}

	Logger.Debug("AMProbe response", zap.Int("200s", count200), zap.Int("total", len(resps)), zap.Int("resps", len(p.ClusterMap)))

	// If we have less than half the expected clusters, try to restart all
	if count200 < len(p.ClusterMap)/2 {
		return p.restart(tryRestartOptions{})
	}

	p.state.LastSuccessfulProbeTime = time.Now()
	p.resetAttempts()
	return nil
}

// resetAttempts resets the state of the task related to attempts
func (p *ProbeTask) resetAttempts() {
	p.state.AttemptedMewldClusterRollingRestart = false
	p.state.AttemptedTargettedKills = []string{}
	p.state.AttemptedKillall = false
	p.state.AttemptedSystemdRestart = false
	p.state.FailedCount = 0
	p.state.BackoffExp = 0
}

type tryRestartOptions struct {
	ProblematicClusters map[uint16][]string // Any specific problematic clusters
}

// restart tries to restart the service only when failedCount is greater than RestartAfterFailed
// (with some other logic such as sending a webhook). Consumers should use restart over tryRestart hence.
func (p *ProbeTask) restart(opts tryRestartOptions) error {
	p.state.FailedCount++

	Logger.Error("Restart called", zap.Int("failedCount", p.state.FailedCount))

	if p.state.FailedCount > p.RestartAfterFailed {
		// Send a webhook to notify
		var webhookContext = discordgo.WebhookParams{
			Content: fmt.Sprintf("%s **CRITICAL ALERT** Restarting service %s due to failed probes (>%d failed probes)", Config.Wafflepaw.RolePing, p.SystemdService, p.RestartAfterFailed),
		}

		_, err := Discord.WebhookExecute(MonitorWebhook.ID, MonitorWebhook.Token, false, &webhookContext)

		if err != nil {
			Logger.Error("Error sending webhook", zap.Error(err))
		}

		err = p.tryRestart(opts)

		if err != nil {
			return fmt.Errorf("error restarting service: %s", err)
		}

		p.state.BackoffExp++ // Increment backoff
	} else {
		_, err := Discord.WebhookExecute(MonitorWebhook.ID, MonitorWebhook.Token, false, &discordgo.WebhookParams{
			Content: fmt.Sprintf("**Failed Probe:** Service %s failed probe check %d/%d failed probes", p.SystemdService, p.state.FailedCount, p.RestartAfterFailed),
		})

		if err != nil {
			Logger.Error("Error sending webhook", zap.Error(err))
		}
	}
	return nil
}

// tryRestart tries to first restart via mewld and return.
//
// If, at the next run, we still go through this function, then we try
// the tryRestartServiceSystemd method
func (p *ProbeTask) tryRestart(opts tryRestartOptions) error {
	Logger.Info("Attempting restart")
	if len(opts.ProblematicClusters) > 0 {
		// Just log for now, restarting individual clusters is not implemented yet
		Logger.Debug("Problematic clusters detected", zap.Any("clusters", opts.ProblematicClusters))

		var hasKilled bool
		for cid, pids := range opts.ProblematicClusters {
			for _, pid := range pids {
				if !slices.Contains(p.state.AttemptedTargettedKills, pid) {
					// try killing it
					Logger.Debug("Attempting to kill problematic cluster", zap.Uint16("clusterId", cid))

					err := p.tryKillPid(pid)

					if err != nil {
						Logger.Error("Error killing pid", zap.Error(err))
					} else {
						hasKilled = true
					}

					p.state.AttemptedTargettedKills = append(p.state.AttemptedTargettedKills, pid)
				}
			}
		}

		if hasKilled {
			// Give some extra buffer time
			time.Sleep(3 * time.Second)

			return nil
		}
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

		return nil
	}

	Logger.Error("Failed to restart service via mewld and systemd")

	_, err := Discord.WebhookExecute(MonitorWebhook.ID, MonitorWebhook.Token, false, &discordgo.WebhookParams{
		Content: fmt.Sprintf("%s **CRITICAL ALERT** Failed to restart service %s via mewld, attempting systemd restart", Config.Wafflepaw.RolePing, p.SystemdService),
	})

	if err != nil {
		Logger.Error("Error sending webhook", zap.Error(err))
	}

	// Give some extra buffer time
	time.Sleep(5 * time.Second)

	return nil
}

// getServiceStatus returns the systemd status of the service
// e.g. "active", "inactive", "failed"
func (p *ProbeTask) getServiceStatus() (string, error) {
	cmd := exec.Command("systemctl", "show", p.SystemdService, "--property=ActiveState", "--value")

	out, err := cmd.Output()

	if err != nil {
		return "", fmt.Errorf("error getting service status: %s", err)
	}

	// Remove newline
	output := strings.Trim(string(out), "\n")

	return output, nil
}

// tryKillPid tries to kill a specific pid
func (p *ProbeTask) tryKillPid(pid string) error {
	cmd := exec.Command("kill", "-9", pid)

	err := cmd.Run()

	if err != nil {
		return fmt.Errorf("error killing pid: %s", err)
	}

	return nil
}

// tryRestartMewldCluster tries to restart a specific cluster with an id
//
// TODO: Implement this
/*func (p *ProbeTask) tryRestartMewldCluster(clusterId uint16) error {
	return nil
}*/

// tryRollingRestartMewldCluster tries to restart the mewld cluster
// using a rolling restart
func (p *ProbeTask) tryRollingRestartMewldCluster() error {
	if p.MewldChannel == "" {
		return nil // This strategy does not work
	}

	rr := mredis.LauncherCmd{
		Scope:     "launcher",
		Action:    "rollingrestart",
		CommandId: crypto.RandString(16),
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
func (p *ProbeTask) tryRestartServiceSystemd() error {
	Logger.Debug("Restarting service", zap.String("service", p.SystemdService))
	cmd := exec.Command("systemctl", "restart", p.SystemdService)

	err := cmd.Run()

	if err != nil {
		return fmt.Errorf("error restarting service [systemd]: %s", err)
	}

	return nil
}
