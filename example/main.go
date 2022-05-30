package main

import (
	"log"
)

func main() {
	app := New()

	log.Fatal(app.Listen())
}
