dev:
	$(MAKE) _dev -j8

_dev: _serve _demo

serve:
	$(MAKE) _serve -j4

_serve: serve.eu-west-1a serve.eu-west-1b serve.eu-west-1c serve.us-west-1a

serve.eu-west-1a:
	cargo run --bin pikav-cli serve -c config/eu-west-1a

serve.eu-west-1b:
	cargo run --bin pikav-cli serve -c config/eu-west-1b

serve.eu-west-1c:
	cargo run --bin pikav-cli serve -c config/eu-west-1c

serve.us-west-1a:
	cargo run --bin pikav-cli serve -c config/us-west-1a

demo:
	$(MAKE) _demo -j4

_demo: demo.eu-west-1a demo.eu-west-1b demo.eu-west-1c demo.us-west-1a

demo.eu-west-1a:
	cd example
	PORT=3001 PIKAV_PORT=6750 cargo run --bin example

demo.eu-west-1b:
	PORT=3002 PIKAV_PORT=6751 cargo run --bin example

demo.eu-west-1c:
	PORT=3003 PIKAV_PORT=6752 cargo run --bin example

demo.us-west-1a:
	PORT=3004 PIKAV_PORT=6753 cargo run --bin example

up:
	docker-compose up -d

stop:
	docker-compose stop

down:
	docker-compose down -v --remove-orphan

clippy:
	cargo clippy --all-features -- -D warnings
