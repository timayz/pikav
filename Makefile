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
	LEPTOS_SITE_ADDR=127.0.0.1:3001 LEPTOS_RELOAD_PORT=3011 PIKAV_API_PORT=6750 PIKAV_PORT=6751 cargo leptos watch --bin-features ssr

demo.eu-west-1b:
	LEPTOS_SITE_ADDR=127.0.0.1:3002 LEPTOS_RELOAD_PORT=3022 PIKAV_API_PORT=6760 PIKAV_PORT=6761 cargo leptos watch --bin-features ssr

demo.us-west-1a:
	LEPTOS_SITE_ADDR=127.0.0.1:3003 LEPTOS_RELOAD_PORT=3033 PIKAV_API_PORT=6770 PIKAV_PORT=6771 cargo leptos watch --bin-features ssr

pub.eu-west-1a:
	cargo run --bin cmd publish -c configs/eu-west-1a

up:
	docker compose up -d --remove-orphans

stop:
	docker compose stop

down:
	docker compose down -v --remove-orphans

clippy:
	cargo clippy --fix --all-features -- -D warnings
	cargo clippy --all-features -- -D warnings

fmt:
	cargo fmt -- --emit files

deny:
	cargo deny check

udeps:
	cargo udeps -p cmd -p pikav-api -p pikav-cluster -p pikav -p pikav-client -p pikav-web

udeps.leptos:
	cargo udeps --features ssr,hydrate -p example

advisory.clean:
	rm -rf ~/.cargo/advisory-db

pants: advisory.clean
	cargo pants

audit: advisory.clean
	cargo audit

outdated:
	cargo outdated

token:
	curl -X POST http://127.0.0.1:6550/oauth/token \
		-H 'Content-Type: application/json' \
		-d '{"client_id": "my_name_is"}'
