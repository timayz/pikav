dev:
	$(MAKE) _dev -j8

_dev: _serve _demo

serve:
	$(MAKE) _serve -j4

_serve: serve.eu-west-1a serve.eu-west-1b serve.us-west-1a

serve.eu-west-1a:
	cargo run --bin cmd serve -c configs/eu-west-1a

serve.eu-west-1b:
	cargo run --bin cmd serve -c configs/eu-west-1b

serve.us-west-1a:
	cargo run --bin cmd serve -c configs/us-west-1a

demo:
	$(MAKE) _demo -j4

_demo: demo.eu-west-1a demo.eu-west-1b demo.us-west-1a

demo.eu-west-1a:
	PORT=3001 PIKAV_PORT=6751 cargo run --bin example

demo.eu-west-1b:
	PORT=3002 PIKAV_PORT=6761 cargo run --bin example

demo.us-west-1a:
	PORT=3003 PIKAV_PORT=6771 cargo run --bin example

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
