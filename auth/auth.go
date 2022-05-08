package auth

import (
	"errors"
	"log"
	"strings"
	"time"

	"github.com/MicahParks/keyfunc"
	"github.com/gofiber/fiber/v2"
	"github.com/golang-jwt/jwt/v4"

	"github.com/timada-org/pikav/config"
)

var jwks *keyfunc.JWKS

func init() {
	options := keyfunc.Options{
		RefreshErrorHandler: func(err error) {
			log.Printf("There was an error with the jwt.Keyfunc\nError: %s", err.Error())
		},
		RefreshInterval:   time.Hour,
		RefreshRateLimit:  time.Minute * 5,
		RefreshTimeout:    time.Second * 10,
		RefreshUnknownKID: true,
	}

	jwks_instance, err := keyfunc.Get(config.AppConfig.JwksURL, options)
	if err != nil {
		log.Fatalf("Failed to create JWKS from resource at the given URL.\nError: %s", err.Error())
	}

	jwks = jwks_instance
}

func GetSessionID(c *fiber.Ctx) string {
	return c.Get("X-Pikav-Session-ID")
}

func GetUserID(c *fiber.Ctx) (string, error) {
	data := strings.Split(c.Get("Authorization"), " ")
	if len(data) != 2 && data[0] != "Bearer" {
		return "", errors.New("invalid authorization http header")
	}

	token, err := jwt.Parse(data[1], jwks.Keyfunc)
	if err != nil {
		return "", errors.New("failed to parse the JWT")
	}

	claims, ok := token.Claims.(jwt.MapClaims)
	if !ok || !token.Valid {
		return "", errors.New("the token is not valid")
	}

	if err := claims.Valid(); err != nil {
		return "", err
	}

	return claims["sub"].(string), nil
}
