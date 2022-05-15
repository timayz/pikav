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

standalone:
	docker-compose -f docker-compose.yml -f docker-compose.standalone.yml up -d

standalone.down:
	docker-compose -f docker-compose.yml -f docker-compose.standalone.yml up -d
