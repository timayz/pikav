run:
	go run ./cmd/pikav

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
	docker-compose -f docker-compose.yml -f docker-compose.standalone.yml pull pikav

standalone.down:
	docker-compose -f docker-compose.yml -f docker-compose.standalone.yml up -d
