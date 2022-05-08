package main

import (
	"log"

	_ "github.com/timada-org/pikav/config"

	"github.com/gofiber/fiber/v2"
	"github.com/timada-org/pikav/auth"
	"github.com/timada-org/pikav/sse"
)

func main() {
	app := fiber.New()

	app.Get("/sse", sse.Handler)

	app.Get("/user-id", func(c *fiber.Ctx) error {
		userId, err := auth.GetUserID(c)
		if err != nil {
			return fiber.ErrUnauthorized
		}

		return c.JSON(fiber.Map{
			"userId": userId,
		})
	})

	// Start server
	log.Fatal(app.Listen(":6750"))
}
