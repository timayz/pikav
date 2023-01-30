dev:
	$(MAKE) _dev -j3

_dev: dev.eu-west-1a dev.eu-west-1b dev.eu-west-1c

dev.eu-west-1a:
	cargo run --bin cmd serve -c configs/eu-west-1a

dev.eu-west-1b:
	cargo run --bin cmd serve -c configs/eu-west-1b

dev.eu-west-1c:
	cargo run --bin cmd serve -c configs/eu-west-1c

pub.eu-west-1a:
	cargo run --bin cmd publish -c configs/eu-west-1a

up:
	docker compose up -d --remove-orphans

stop:
	docker compose stop

down:
	docker compose down -v --remove-orphans

clippy:
	cargo clippy --all-features -- -D warnings

fmt:
	cargo fmt -- --emit files

token:
	curl -X POST http://127.0.0.1:6550/oauth/token \
		-H 'Content-Type: application/json' \
		-d '{"client_id": "my_name_is"}'
