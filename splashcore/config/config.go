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
	DiscordAuth        DiscordAuth         `yaml:"discord_auth" validate:"required"`
	Sites              Sites               `yaml:"sites" validate:"required"`
	Channels           Channels            `yaml:"channels" validate:"required"`
	Roles              Roles               `yaml:"roles" validate:"required"`
	JAPI               JAPI                `yaml:"japi" validate:"required"`
	Notifications      Notifications       `yaml:"notifications" validate:"required"`
	Servers            Servers             `yaml:"servers" validate:"required"`
	Meta               Meta                `yaml:"meta" validate:"required"`
	ObjectStorage      ObjectStorageConfig `yaml:"object_storage" validate:"required"`
	SimpleGatewayProxy SimpleGatewayProxy  `yaml:"simple_gateway_proxy" validate:"required"`
	SurrealDB          SurrealDB           `yaml:"surreal" validate:"required"`
}

type SimpleGatewayProxy struct {
	Port int    `yaml:"port" default:"3220" comment:"Port to run the proxy on" validate:"required"`
	Url  string `yaml:"url" default:"http://localhost:3220" comment:"Url proxy is accessible on" validate:"required"`
}

type SurrealDB struct {
	Url      string `yaml:"url" comment:"Surreal Cache Url" validate:"required"`
	Username string `yaml:"username" comment:"Surreal Cache Username" validate:"required"`
	Password string `yaml:"password" comment:"Surreal Cache Password" validate:"required"`
}

type DiscordAuth struct {
	Token            string   `yaml:"token" comment:"Discord bot token" validate:"required"`
	ClientID         string   `yaml:"client_id" default:"849331145862283275" comment:"Discord Client ID" validate:"required"`
	ClientSecret     string   `yaml:"client_secret" comment:"Discord Client Secret" validate:"required"`
	AllowedRedirects []string `yaml:"allowed_redirects" default:"http://localhost:3000/auth" validate:"required"`
	MewldRedirect    string   `yaml:"mewld_redirect" default:"https://mewld.antiraid.xyz/login" validate:"required"`
	CanUseBot        []string `yaml:"can_use_bot" default:"728871946456137770,564164277251080208,775855009421066262" validate:"required"`
	RootUsers        []string `yaml:"root_users" default:"728871946456137770,564164277251080208,775855009421066262" validate:"required"`
}

type Sites struct {
	Frontend Differs[string] `yaml:"frontend" default:"https://antiraid.xyz" comment:"Frontend URL" validate:"required"`
	API      Differs[string] `yaml:"api" default:"https://splashtail.antiraid.xyz" comment:"API URL" validate:"required"`
	CDN      string          `yaml:"cdn" default:"https://cdn.antiraid.xyz" comment:"CDN URL" validate:"required"`
	Instatus string          `yaml:"instatus" default:"https://status.antiraid.xyz" comment:"Instatus Status Page URL" validate:"required"`
}

type Roles struct {
}

type Channels struct {
}

type JAPI struct {
	Key string `yaml:"key" default:"Currently unused, ignore this field" comment:"JAPI Key. Get it from https://japi.rest" validate:"required"`
}

type Notifications struct {
	VapidPublicKey  string `yaml:"vapid_public_key" default:"BNMhOWvN-6_jm72D3Ncnxmvwz03TLDNVOi1bd9uD-OjWbHmaa4w1A5nq8MTjSKL_tnMueI64ZxtRXWRltRu0Vio" comment:"Vapid Public Key (https://www.stephane-quantin.com/en/tools/generators/vapid-keys)" validate:"required"`
	VapidPrivateKey string `yaml:"vapid_private_key" default:"Set this here if you want notifications to work" comment:"Vapid Private Key (https://www.stephane-quantin.com/en/tools/generators/vapid-keys)" validate:"required"`
}

type Servers struct {
	Main string `yaml:"main" default:"1064135068928454766" comment:"Main Server ID" validate:"required"`
}

type Meta struct {
	AnimusMagicChannel   Differs[string] `yaml:"animus_magic_channel" default:"animus_magic_staging" comment:"Animus Magic Channel" validate:"required"`
	WebDisableRatelimits bool            `yaml:"web_disable_ratelimits" comment:"Disable ratelimits for the web server"`
	PostgresURL          string          `yaml:"postgres_url" default:"postgresql:///antiraid" comment:"Postgres URL" validate:"required"`
	BotRedisURL          string          `yaml:"bot_redis_url" default:"redis://localhost:6379/0" comment:"Bot Redis URL" validate:"required"`
	RedisURL             Differs[string] `yaml:"redis_url" default:"redis://localhost:6379" comment:"Redis URL" validate:"required"`
	JobserverPort        Differs[int]    `yaml:"jobserver_port" default:"8080" comment:"Jobserver Port" validate:"required"`
	Port                 Differs[int]    `yaml:"port" default:":8081" comment:"Port to run the server on" validate:"required"`
	CDNPath              string          `yaml:"cdn_path" default:"/failuremgmt/cdn/antiraid" comment:"CDN Path" validate:"required"`
	VulgarList           []string        `yaml:"vulgar_list" default:"fuck,suck,shit,kill" validate:"required"`
	UrgentMentions       string          `yaml:"urgent_mentions" default:"<@&1061643797315993701>" comment:"Urgent mentions" validate:"required"`
	Proxy                string          `yaml:"proxy" default:"http://127.0.0.1:3219" comment:"Popplio Proxy URL" validate:"required"`
	DPSecret             string          `yaml:"dp_secret" comment:"DeployProxy Auth URL for super-sensitive pages" validate:"required"`
	DebugTaskLogger      bool            `yaml:"debug_task_logger" comment:"Debug the task logger"`
	SupportServer        string          `yaml:"support_server" comment:"Discord Support Server Link" default:"https://discord.gg/u78NFAXm" validate:"required"`
}

type BotList struct {
	Name       string         `yaml:"name" comment:"Bot List Name" validate:"required"`
	APIUrl     string         `yaml:"api_url" comment:"API Url for the list" validate:"required"`
	APIToken   string         `yaml:"api_token" comment:"API Token for the list" validate:"required"`
	AuthFormat string         `yaml:"auth_format" comment:"Can be one of h#[header]/{token} or u#[token]={token} or b#[key]={token} (brackets means that anything can be substituted in)" validate:"required"`
	PostStats  *BotListAction `yaml:"post_stats" comment:"Post Stats Action"`
}

type BotListAction struct {
	Enabled    bool              `yaml:"enabled" comment:"Whether or not the action is enabled or not" validate:"required"`
	Method     string            `yaml:"method" comment:"What HTTP method to use"`
	Interval   int64             `yaml:"interval" comment:"What interval to send messages at"`
	URLFormat  string            `yaml:"url_format" comment:"Must be u#{url}?[key1]={key2} (brackets means that anything can be substituted in)"`
	DataFormat map[string]string `yaml:"data_format" comment:"Must be {key1}={key2} (brackets means that anything can be substituted in)"`
}

// Some data such as backups can get quite large.
// These are stored on a S3-like bucket such as DigitalOcean spaces
type ObjectStorageConfig struct {
	Type        string `yaml:"type" comment:"Must be one of s3-like or local" validate:"required" oneof:"s3-like local"`
	Path        string `yaml:"path" comment:"If s3-like, this should be the name of the bucket. Otherwise, should be the path to the location to store to"`
	Endpoint    string `yaml:"endpoint" comment:"Only for s3-like, this should be the endpoint to the bucket."`
	CdnEndpoint string `yaml:"cdn_endpoint" comment:"Only for s3-like (and DigitalOcean mainly), this should be the CDN endpoint to the bucket."`
	Secure      bool   `yaml:"secure" comment:"Only for s3-like, this should be whether or not to use a secure connection to the bucket."`
	CdnSecure   bool   `yaml:"cdn_secure" comment:"Only for s3-like, this should be whether or not to use a secure connection to the CDN."`
	AccessKey   string `yaml:"access_key" comment:"Only for s3-like, this should be the access key to the bucket."`
	SecretKey   string `yaml:"secret_key" comment:"Only for s3-like, this should be the secret key to the bucket."`
}
