package serve

import (
	"fmt"
	"os"

	"github.com/ilyakaznacheev/cleanenv"
	"github.com/spf13/cobra"
	"github.com/timada-org/pikav/internal/api"
	"github.com/timada-org/pikav/internal/core"
	"github.com/timada-org/pikav/internal/sse"
)

type ServeConfig struct {
	Cluster struct {
		Addr string `yaml:"addr" env:"PIKAV_CLUSTER_ADDRE"`
	} `yaml:"cluster"`
	Api struct {
		Addr string `yaml:"addr" env:"PIKAV_API_ADDR"`
	} `yaml:"api"`
	Jwks struct {
		Url string `yaml:"url" env:"PIKAV_JWKS_URL"`
	} `yaml:"jwks"`
	Nodes []string
}

func NewServeCmd() (cmd *cobra.Command) {
	var configPath string

	cmd = &cobra.Command{
		Use:   "serve",
		Short: "Run timada pikav server",
		Run: func(cmd *cobra.Command, args []string) {
			var cfg ServeConfig

			if err := cleanenv.ReadConfig(configPath, &cfg); err != nil {
				fmt.Fprintln(os.Stderr, err)
				os.Exit(1)
			}

			sseServer := sse.New()

			bus := core.NewEventBus(&core.EventBusOptions{
				Server: sseServer,
			})

			server := api.NewServer(api.ServerOptions{
				Addr:    cfg.Api.Addr,
				Sse:     sseServer,
				Bus:     bus,
				JwksURL: cfg.Jwks.Url,
			})

			server.Start()
		},
	}

	cmd.Flags().StringVarP(&configPath, "config", "c", "", "config file path")

	if err := cmd.MarkFlagRequired("config"); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	return cmd
}
