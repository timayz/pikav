package cmd

import (
	"log"

	"github.com/spf13/cobra"
	"github.com/timada-org/pikav/internal/api"
	"github.com/timada-org/pikav/internal/core"
)

var (
	cfgFile string

	serveCmd = &cobra.Command{
		Use:   "serve",
		Short: "Run the pikav server",

		Run: func(cmd *cobra.Command, args []string) {
			config, err := core.NewConfig(cfgFile)
			if err != nil {
				log.Fatalln(err)
			}

			app := api.New(config)
			defer app.Close()

			log.Fatal(app.Listen())
		},
	}
)

func init() {
	serveCmd.PersistentFlags().StringVarP(&cfgFile, "config", "c", "/etc/config/pikav.yaml", "config file (default is /etc/config/pikav.yaml)")
}
