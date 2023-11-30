package config

import (
	_ "embed"
	"strings"
)

const (
	CurrentEnvProd    = "prod"
	CurrentEnvStaging = "staging"
)

//go:embed current-env
var CurrentEnv string

func init() {
	CurrentEnv = strings.TrimSpace(CurrentEnv)

	if CurrentEnv != CurrentEnvProd && CurrentEnv != CurrentEnvStaging {
		panic("invalid environment")
	}
}

// Common struct for values that differ between staging and production environments
type Differs[T any] struct {
	Staging T `yaml:"staging" comment:"Staging value" validate:"required"`
	Prod    T `yaml:"prod" comment:"Production value" validate:"required"`
}

func (d *Differs[T]) Parse() T {
	if CurrentEnv == CurrentEnvProd {
		return d.Prod
	} else if CurrentEnv == CurrentEnvStaging {
		return d.Staging
	} else {
		panic("invalid environment")
	}
}

func (d *Differs[T]) Production() T {
	return d.Prod
}

type Config struct {
	DiscordAuth   DiscordAuth   `yaml:"discord_auth" validate:"required"`
	Sites         Sites         `yaml:"sites" validate:"required"`
	Channels      Channels      `yaml:"channels" validate:"required"`
	Roles         Roles         `yaml:"roles" validate:"required"`
	JAPI          JAPI          `yaml:"japi" validate:"required"`
	Notifications Notifications `yaml:"notifications" validate:"required"`
	Servers       Servers       `yaml:"servers" validate:"required"`
	Meta          Meta          `yaml:"meta" validate:"required"`
	Hcaptcha      Hcaptcha      `yaml:"hcaptcha" validate:"required"`
}

type Hcaptcha struct {
	SiteKey string `yaml:"site_key" comment:"Hcaptcha Site Key" validate:"required"`
	Secret  string `yaml:"secret" comment:"Hcaptcha Secret" validate:"required"`
}

type DiscordAuth struct {
	Token            string   `yaml:"token" comment:"Discord bot token" validate:"required"`
	ClientID         string   `yaml:"client_id" default:"815553000470478850" comment:"Discord Client ID" validate:"required"`
	ClientSecret     string   `yaml:"client_secret" comment:"Discord Client Secret" validate:"required"`
	AllowedRedirects []string `yaml:"allowed_redirects" default:"http://localhost:3000/auth/sauron,http://localhost:8000/auth/sauron,https://reedwhisker.infinitybots.gg/auth/sauron,https://infinitybots.gg/auth/sauron,https://botlist.site/auth/sauron,https://infinitybots.xyz/auth/sauron" validate:"required"`
}

type Sites struct {
	Frontend Differs[string] `yaml:"frontend" default:"https://reedwhisker.infinitybots.gg" comment:"Frontend URL" validate:"required"`
	API      Differs[string] `yaml:"api" default:"https://spider.infinitybots.gg" comment:"API URL" validate:"required"`
	CDN      string          `yaml:"cdn" default:"https://cdn.infinitybots.gg" comment:"CDN URL" validate:"required"`
	Instatus string          `yaml:"instatus" default:"https://infinity-bots.instatus.com" comment:"Instatus Status Page URL" validate:"required"`
}

type Roles struct {
}

type Channels struct {
}

type JAPI struct {
	Key string `yaml:"key" comment:"JAPI Key. Get it from https://japi.rest" validate:"required"`
}

type Notifications struct {
	VapidPublicKey  string `yaml:"vapid_public_key" default:"BIdUNSqYzqVjbdJhn8WK6SDYDVj85mKtctrEgj14KkjxIMerxQ9wywvvxECkuP8rL3s8zDgZSE9HSqW1wmhVPM8" comment:"Vapid Public Key (https://www.stephane-quantin.com/en/tools/generators/vapid-keys)" validate:"required"`
	VapidPrivateKey string `yaml:"vapid_private_key" comment:"Vapid Private Key (https://www.stephane-quantin.com/en/tools/generators/vapid-keys)" validate:"required"`
}

type Servers struct {
	Main string `yaml:"main" default:"758641373074423808" comment:"Main Server ID" validate:"required"`
}

type Meta struct {
	PostgresURL         string          `yaml:"postgres_url" default:"postgresql:///infinity" comment:"Postgres URL" validate:"required"`
	RedisURL            Differs[string] `yaml:"redis_url" default:"redis://localhost:6379" comment:"Redis URL" validate:"required"`
	Port                Differs[string] `yaml:"port" default:":8081" comment:"Port to run the server on" validate:"required"`
	CDNPath             string          `yaml:"cdn_path" default:"/failuremgmt/cdn/antiraid" comment:"CDN Path" validate:"required"`
	SecureStorage       string          `yaml:"secure_storage" default:"/failuremgmt/sec/antiraid" comment:"Blob Storage URL" validate:"required"`
	VulgarList          []string        `yaml:"vulgar_list" default:"fuck,suck,shit,kill" validate:"required"`
	UrgentMentions      string          `yaml:"urgent_mentions" default:"<@&1061643797315993701>" comment:"Urgent mentions" validate:"required"`
	UptimeRobotROAPIKey string          `yaml:"uptime_robot_ro_api_key" default:"" comment:"Uptime Robot Read-Only API Key" validate:"required"`
	PopplioProxy        string          `yaml:"popplio_proxy" default:"http://127.0.0.1:3219" comment:"Popplio Proxy URL" validate:"required"`
}
