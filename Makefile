dev:
	$(MAKE) _dev -j3

_dev: dev.eu-west-1a dev.eu-west-1b dev.eu-west-1c

dev.eu-west-1a:
	cargo run --bin pikav-cli serve -c configs/eu-west-1a

dev.eu-west-1b:
	cargo run --bin pikav-cli serve -c configs/eu-west-1b

dev.eu-west-1c:
	cargo run --bin pikav-cli serve -c configs/eu-west-1c

up:
	docker-compose up -d --remove-orphan

stop:
	docker-compose stop

down:
	docker-compose down -v --remove-orphan

clippy:
	cargo clippy --all-features -- -D warnings

fmt:
	cargo fmt -- --emit files
