FROM golang:1.18 as build-env

WORKDIR /go/src/app

COPY go_pkg_mod ~/go/pkg/mod
COPY . .

RUN CGO_ENABLED=0 go build -o /go/bin/app ./cmd/pikav

FROM gcr.io/distroless/static

ENV PIKAV_CONFIG_DIR=/etc/pikav/

EXPOSE 6750

COPY --from=build-env /go/bin/app /
COPY --from=build-env /go/src/app/configs/config.yml $PIKAV_CONFIG_DIR
CMD ["/app"]
