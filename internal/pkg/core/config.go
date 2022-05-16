package core

import (
	"fmt"
	"os"

	"github.com/gookit/config/v2"
	"github.com/gookit/config/v2/yaml"
)

type Broker struct {
	URL   string `config:"url"`
	Topic string `config:"topic"`
}

type Config struct {
	ID      string `config:"id"`
	Addr    string `config:"addr"`
	JwksURL string `config:"jwks_url"`
	Broker  Broker `config:"broker"`
}

func NewConfig() (*Config, error) {
	var appConfig Config

	config.WithOptions(func(opt *config.Options) {
		opt.ParseEnv = true
		opt.DecoderConfig.TagName = "config"
	})

	config.AddDriver(yaml.Driver)

	baseDir := "configs/"

	if p := os.Getenv("PIKAV_CONFIG_DIR"); p != "" {
		baseDir = p
	}

	if err := config.LoadFiles(fmt.Sprintf("%sconfig.yml", baseDir)); err != nil {
		return nil, err
	}

	if err := config.LoadExists(fmt.Sprintf("%sconfig.local.yml", baseDir)); err != nil {
		return nil, err
	}

	if err := config.BindStruct("", &appConfig); err != nil {
		return nil, err
	}

	return &appConfig, nil
}
