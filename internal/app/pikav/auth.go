package pikav

import (
	"errors"
	"log"
	"net/http"
	"strings"
	"time"

	"github.com/MicahParks/keyfunc"
	"github.com/golang-jwt/jwt/v4"
)

type Auth struct{ jwks keyfunc.JWKS }

func newAuth(url string) *Auth {
	options := keyfunc.Options{
		RefreshErrorHandler: func(err error) {
			log.Printf("There was an error with the jwt.Keyfunc\nError: %s", err.Error())
		},
		RefreshInterval:   time.Hour,
		RefreshRateLimit:  time.Minute * 5,
		RefreshTimeout:    time.Second * 10,
		RefreshUnknownKID: true,
	}

	jwks, err := keyfunc.Get(url, options)
	if err != nil {
		log.Fatalf("Failed to create JWKS from resource at the given URL.\nError: %s", err.Error())
	}

	return &Auth{*jwks}
}

func (auth *Auth) sessionID(r *http.Request) string {
	return r.Header.Get("X-Pikav-Session-ID")
}

func (auth *Auth) userID(r *http.Request) (string, error) {
	data := strings.Split(r.Header.Get("Authorization"), " ")
	if len(data) != 2 && data[0] != "Bearer" {
		return "", errors.New("invalid authorization http header")
	}

	token, err := jwt.Parse(data[1], auth.jwks.Keyfunc)
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
