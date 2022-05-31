dev:
	$(MAKE) _dev -j8

_dev: _serve _demo

serve: 
	$(MAKE) _serve -j4

_serve: serve.eu-west-1a serve.eu-west-1b serve.eu-west-1c serve.us-west-1a

serve.eu-west-1a:
	go run . serve -c configs/eu-west-1a.yml

serve.eu-west-1b:
	go run . serve -c configs/eu-west-1b.yml

serve.eu-west-1c:
	go run . serve -c configs/eu-west-1c.yml

serve.us-west-1a:
	go run . serve -c configs/us-west-1a.yml

demo: 
	$(MAKE) _demo -j4

_demo: demo.eu-west-1a demo.eu-west-1b demo.eu-west-1c demo.us-west-1a

demo.eu-west-1a:
	PORT=:3001 PIKAV_PORT=6750 go run ./example

demo.eu-west-1b:
	PORT=:3002 PIKAV_PORT=6751 go run ./example

demo.eu-west-1c:
	PORT=:3003 PIKAV_PORT=6752 go run ./example

demo.us-west-1a:
	PORT=:3004 PIKAV_PORT=6753 go run ./example

lint:
	docker run --rm -v $(shell pwd):/app -w /app golangci/golangci-lint:v1.45.2 golangci-lint run -v

up:
	docker-compose up -d --remove-orphan

stop:
	docker-compose stop

down:
	docker-compose down -v --remove-orphan

standalone: standalone.pull
	docker-compose -f docker-compose.yml -f docker-compose.standalone.yml up -d

standalone.pull:
	docker-compose -f docker-compose.yml -f docker-compose.standalone.yml pull pikav-eu-west-1a

standalone.down:
	docker-compose -f docker-compose.yml -f docker-compose.standalone.yml up -d

standalone.restart:
	docker-compose -f docker-compose.yml -f docker-compose.standalone.yml restart
