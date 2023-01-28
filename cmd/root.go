package cmd

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
	"github.com/timada-org/pikav/cmd/migrate"
	"github.com/timada-org/pikav/cmd/serve"
)

func NewRootCmd() (cmd *cobra.Command) {
	cmd = &cobra.Command{
		Use: "pikav",
	}

	cmd.AddCommand(serve.NewServeCmd())
	cmd.AddCommand(migrate.NewMigrateCmd())

	return cmd
}

func Execute() {
	c := NewRootCmd()

	if err := c.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
