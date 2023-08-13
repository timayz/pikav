Pikav is a simple cloud native SSE server with topic subscription

---

## Getting Started

### Docker compose

```yaml
version: "3.9"

services:
  pikav-eu-west-1a:
    image: timayz/pikav
    command: serve -c /etc/config/pikav.yml
    ports:
      - "6750:6750"
    depends_on:
      - oathkeeper
    volumes:
      - ./.docker/pikav/eu-west-1a.yml:/etc/config/pikav.yml
    networks:
      default:
        aliases:
          - eu-west-1a.pikav.internal

  pikav-eu-west-1b:
    image: timayz/pikav
    command: serve -c /etc/config/pikav.yml
    ports:
      - "6751:6750"
    depends_on:
      - oathkeeper
    volumes:
      - ./.docker/pikav/eu-west-1b.yml:/etc/config/pikav.yml
    networks:
      default:
        aliases:
          - eu-west-1b.pikav.internal

  pikav-us-west-1a:
    image: timayz/pikav
    command: serve -c /etc/config/pikav.yml
    ports:
      - "6752:6750"
    depends_on:
      - oathkeeper
    volumes:
      - ./.docker/pikav/us-west-1a.yml:/etc/config/pikav.yml
    networks:
      default:
        aliases:
          - us-west-1a.pikav.internal

  pikav-us-west-1a:
    image: timayz/pikav
    command: serve -c /etc/config/pikav.yml
    ports:
      - "6753:6750"
    depends_on:
      - oathkeeper
    volumes:
      - ./.docker/pikav/us-west-1a.yml:/etc/config/pikav.yml
    networks:
      default:
        aliases:
          - us-west-1a.pikav.internal
```

### Config

```yaml
listen: "0.0.0.0:6750"

cors_permissive: true

jwks_url: http://127.0.0.1:4456/.well-known/jwks.json

nodes:
  - url: http://127.0.0.1:6751
    shared: true
  - url: http://127.0.0.1:6752
    shared: true
  - url: http://127.0.0.1:6753

```