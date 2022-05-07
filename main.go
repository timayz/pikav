package main

import (
	"log"

	"github.com/gofiber/fiber/v2"
	"github.com/timada-org/pikav/sse"
)

var index = []byte(`<!DOCTYPE html>
<html>
<body>
<h1>SSE Messages</h1>
<div id="result"></div>
<script>
if(typeof(EventSource) !== "undefined") {
  var source = new EventSource("http://localhost:6750/sse");
  source.onmessage = function(event) {
    document.getElementById("result").innerHTML += event.data + "<br>";
  };
} else {
  document.getElementById("result").innerHTML = "Sorry, your browser does not support server-sent events...";
}
</script>
</body>
</html>
`)

func main() {
	sseServer := sse.New()
	app := fiber.New()

	app.Get("/", func(c *fiber.Ctx) error {
		c.Response().Header.SetContentType(fiber.MIMETextHTMLCharsetUTF8)

		return c.Status(fiber.StatusOK).Send(index)
	})

	app.Get("/sse", sseServer.Handler)

	// Start server
	log.Fatal(app.Listen(":6750"))
}
