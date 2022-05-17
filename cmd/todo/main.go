package main

import (
	"log"

	"github.com/timada-org/pikav/internal/app/todo"
)

func main() {
	app := todo.New()

	log.Fatal(app.Listen())
}
