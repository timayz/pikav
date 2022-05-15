package main

import (
	"log"

	"github.com/timada-org/pikav/internal/app/pikav"
)

func main() {
	app := pikav.New()
	defer app.Close()

	log.Fatal(app.Listen())
}
