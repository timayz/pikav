package main

import (
	"github.com/spf13/cobra"
	"github.com/timada-org/pikav/pkg/client"
	"github.com/timada-org/pikav/pkg/topic"
)

func main() {
	rootCmd := &cobra.Command{
		Use:   "pikav-cli",
		Short: "Cli for pikav",
		Run: func(cmd *cobra.Command, args []string) {
			c, _ := client.New(client.ClientOptions{
				URL: "http://127.0.0.1:6750/pub",
			})

			name, _ := topic.NewName("todos")

			_ = c.Send(&client.Event{
				UserID: "9eac6c3d-d242-48ad-a2e0-52ada6f1358f",
				Topic:  name,
				Name:   "Created",
				Data: map[string]string{
					"hello": "my friend",
				},
			})
		},
	}

	_ = rootCmd.Execute()
}
