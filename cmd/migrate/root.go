package migrate

import (
	"github.com/spf13/cobra"
)

func NewMigrateCmd() (cmd *cobra.Command) {
	cmd = &cobra.Command{
		Use:   "migrate",
		Short: "Various migration helpers",
		Run: func(cmd *cobra.Command, args []string) {

		},
	}

	return cmd
}
