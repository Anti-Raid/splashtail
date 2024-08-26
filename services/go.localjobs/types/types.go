package types

type Config struct {
	BotToken string
	Settings map[string]string
	Args     map[string]string `yaml:"-"` // Dynamically set
}
