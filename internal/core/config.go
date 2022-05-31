package core

import (
	"strings"

	"github.com/gookit/config/v2"
	"github.com/gookit/config/v2/yaml"
)

type Node struct {
	URL    string `config:"url"`
	Shared string `config:"shared"`
}

type Config struct {
	Addr    string `config:"addr"`
	JwksURL string `config:"jwks_url"`
	Nodes   []Node `config:"nodes"`
}

func NewConfig(path string) (*Config, error) {
	var appConfig Config

	config.WithOptions(func(opt *config.Options) {
		opt.ParseEnv = true
		opt.DecoderConfig.TagName = "config"
	})

	config.AddDriver(yaml.Driver)

	if err := config.LoadFiles(path); err != nil {
		return nil, err
	}

	if err := config.LoadExists(strings.Replace(path, ".yml", ".local.yml", 1)); err != nil {
		return nil, err
	}

	if err := config.BindStruct("", &appConfig); err != nil {
		return nil, err
	}

	return &appConfig, nil
}
