dev:
	$(MAKE) _dev -j4

_dev: dev.eu-west-1a dev.eu-west-1b dev.eu-west-1c

dev.eu-west-1a:
	go run . serve -c configs/eu-west-1a.yml

dev.eu-west-1b:
	go run . serve -c configs/eu-west-1b.yml

dev.eu-west-1c:
	go run . serve -c configs/eu-west-1c.yml

up:
	docker compose up -d --remove-orphan

stop:
	docker compose stop

down:
	docker compose down -v --remove-orphan

# standalone:
# 	docker-compose -f docker-compose.yml -f docker-compose.standalone.yml pull
# 	docker-compose -f docker-compose.yml -f docker-compose.standalone.yml up -d --remove-orphan

# standalone.stop:
# 	docker-compose -f docker-compose.yml -f docker-compose.standalone.yml stop

# standalone.down:
# 	docker-compose -f docker-compose.yml -f docker-compose.standalone.yml down -v --remove-orphan

# standalone.logs:
# 	docker-compose -f docker-compose.yml -f docker-compose.standalone.yml logs -f

lint:
	golangci-lint run
