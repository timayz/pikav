package cmd

import "github.com/spf13/cobra"

var (
	rootCmd = &cobra.Command{
		Use:   "pikav",
		Short: "A simple SSE with topic subscription",
		Long:  `Pikav is a server sent event that allow client to subscribe topic using MQTT Topics, Wildcards`,
	}
)

func Execute() error {
	return rootCmd.Execute()
}

func init() {
	rootCmd.AddCommand(serveCmd)
}
