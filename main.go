package main

import (
	"os"

	"github.com/timada-org/pikav/cmd"
)

func main() {
	if err := cmd.Execute(); err != nil {
		os.Exit(1)
	}
}
