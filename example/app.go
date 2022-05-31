package main

import (
	"encoding/json"
	"fmt"
	"log"
	"math/rand"
	"net/http"
	"os"
	"strconv"
	"time"

	"github.com/glebarez/sqlite"
	"github.com/julienschmidt/httprouter"
	"github.com/timada-org/pikav/internal/api"
	"github.com/timada-org/pikav/internal/core"
	"github.com/timada-org/pikav/pkg/client"
	"github.com/timada-org/pikav/pkg/topic"
	"gorm.io/gorm"
)

type App struct {
	auth *api.Auth
	db   *gorm.DB
	c    *client.Client
}

func New() *App {

	auth, err := api.NewAuth("http://127.0.0.1:4456/.well-known/jwks.json")
	if err != nil {
		log.Fatalln(err)
	}

	db, err := gorm.Open(sqlite.Open("gorm.db"), &gorm.Config{})
	if err != nil {
		log.Fatalln(err)
	}

	if err := db.AutoMigrate(&Todo{}); err != nil {
		log.Fatalln(err)
	}

	c, err := client.New(client.ClientOptions{
		URL: fmt.Sprintf("http://127.0.0.1:%s/pub", os.Getenv("PIKAV_PORT")),
	})

	if err != nil {
		log.Fatalln(err)
	}

	app := &App{
		auth,
		db,
		c,
	}

	return app
}

func (app *App) Listen() error {
	router := httprouter.New()
	router.GET("/todos", app.list())
	router.POST("/todos", app.create())
	router.PUT("/todos/:id", app.update())
	router.DELETE("/todos/:id", app.delete())

	log.Printf("Listening on %s", os.Getenv("PORT"))

	return http.ListenAndServe(os.Getenv("PORT"), router)
}

func (app *App) list() httprouter.Handle {
	return func(w http.ResponseWriter, r *http.Request, p httprouter.Params) {
		userID, err := app.auth.UserID(r)
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		var todos []Todo
		if err := app.db.Where("user_id = ?", userID).Find(&todos).Error; err != nil {
			http.Error(w, "Internal server error.", http.StatusInternalServerError)
			return
		}

		todosB, _ := json.Marshal(todos)

		w.Header().Add("Content-Type", "application/json")

		if _, err := w.Write(todosB); err != nil {
			log.Println(err.Error())
			return
		}
	}
}

type CreateInput struct {
	Text string `json:"text"`
}

func (app *App) create() httprouter.Handle {
	return func(w http.ResponseWriter, r *http.Request, p httprouter.Params) {
		userID, err := app.auth.UserID(r)
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		decoder := json.NewDecoder(r.Body)
		var input CreateInput
		if err := decoder.Decode(&input); err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		todo := Todo{Text: input.Text, Done: false, UserID: userID}

		if err := app.db.Create(&todo).Error; err != nil {
			http.Error(w, "Internal Server Error.", http.StatusInternalServerError)
			return
		}

		go func() {
			n := rand.Intn(3)
			time.Sleep(time.Duration(n) * time.Second)

			topic, _ := topic.NewName(fmt.Sprintf("todos/%d", todo.ID))

			_ = app.c.Send(&core.Event{
				UserID: userID,
				Topic:  topic,
				Name:   "Created",
				Data:   todo,
			})
		}()

		w.Header().Add("Content-Type", "application/json")

		if _, err := w.Write([]byte("{\"success\": true}")); err != nil {
			log.Println(err.Error())
			return
		}
	}
}

type UpdateInput struct {
	Text string `json:"text"`
	Done bool   `json:"done"`
}

func (app *App) update() httprouter.Handle {
	return func(w http.ResponseWriter, r *http.Request, p httprouter.Params) {
		userID, err := app.auth.UserID(r)
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		id, err := strconv.ParseUint(p.ByName("id"), 0, 64)
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		var todo Todo

		if err := app.db.Where(&Todo{UserID: userID, ID: id}).First(&todo).Error; err != nil {
			http.Error(w, "Not found.", http.StatusNotFound)
			return
		}

		decoder := json.NewDecoder(r.Body)
		var input UpdateInput
		if err := decoder.Decode(&input); err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		todo.Text = input.Text
		todo.Done = input.Done

		if err := app.db.Save(&todo).Error; err != nil {
			http.Error(w, "Internal server error.", http.StatusInternalServerError)
			return
		}

		go func() {
			n := rand.Intn(3)
			time.Sleep(time.Duration(n) * time.Second)

			topic, _ := topic.NewName(fmt.Sprintf("todos/%d", todo.ID))

			_ = app.c.Send(&core.Event{
				UserID: userID,
				Topic:  topic,
				Name:   "Updated",
				Data:   todo,
			})
		}()

		w.Header().Add("Content-Type", "application/json")

		if _, err := w.Write([]byte("{\"success\": true}")); err != nil {
			log.Println(err.Error())
			return
		}
	}
}

func (app *App) delete() httprouter.Handle {
	return func(w http.ResponseWriter, r *http.Request, p httprouter.Params) {
		userID, err := app.auth.UserID(r)
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		id, err := strconv.ParseUint(p.ByName("id"), 0, 64)
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		var todo Todo

		if err := app.db.Where(&Todo{UserID: userID, ID: id}).First(&todo).Error; err != nil {
			http.Error(w, "Not found.", http.StatusNotFound)
			return
		}

		if err := app.db.Delete(&todo).Error; err != nil {
			http.Error(w, "Internal server error.", http.StatusInternalServerError)
			return
		}

		go func() {
			n := rand.Intn(3)
			time.Sleep(time.Duration(n) * time.Second)

			topic, _ := topic.NewName(fmt.Sprintf("todos/%d", todo.ID))

			_ = app.c.Send(&core.Event{
				UserID: userID,
				Topic:  topic,
				Name:   "Deleted",
				Data: map[string]any{
					"id": todo.ID,
				},
			})
		}()

		w.Header().Add("Content-Type", "application/json")

		if _, err := w.Write([]byte("{\"success\": true}")); err != nil {
			log.Println(err.Error())
			return
		}
	}
}
