package config

import (
	"fmt"
	"os"

	"github.com/gookit/config/v2"
	"github.com/gookit/config/v2/yaml"
)

type Config struct {
	JwksURL string `config:"jwks_url"`
}

var AppConfig Config

func init() {
	config.WithOptions(func(opt *config.Options) {
		opt.ParseEnv = true
		opt.DecoderConfig.TagName = "config"
	})

	config.AddDriver(yaml.Driver)

	baseDir := os.Getenv("PIKAV_CONFIG_DIR")

	if err := config.LoadFiles(fmt.Sprintf("%sconfig.yml", baseDir)); err != nil {
		panic(err)
	}

	if err := config.LoadExists(fmt.Sprintf("%sconfig.local.yml", baseDir)); err != nil {
		panic(err)
	}

	if err := config.BindStruct("", &AppConfig); err != nil {
		panic(err)
	}

}
