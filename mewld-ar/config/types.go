package config

type Oauth struct {
	ClientID     string `yaml:"client_id"`
	ClientSecret string `yaml:"client_secret" json:"-"`
	RedirectURL  string `yaml:"redirect_url"`
}

type CoreConfig struct {
	Token        string   `yaml:"token"` // ANTIRAID-SPECIFIC: Add token to config
	Dir          string   `yaml:"dir"`
	OverrideDir  string   `yaml:"override_dir"`
	Names        []string `yaml:"names"`
	Module       string   `yaml:"module"`
	Redis        string   `yaml:"redis"`
	RedisChannel string   `yaml:"redis_channel"`
	Interp       string   `yaml:"interp"`
	AllowedIDS   []string `yaml:"allowed_ids"`
	Oauth        Oauth    `yaml:"oauth"`
	PingInterval int      `yaml:"ping_interval"`
	PerCluster   uint64   `yaml:"per_cluster"`
}

// ANTIRAID-SPECIFIC: Remove Env from CoreConfig
