FROM golang:1.18-alpine3.15 as build-env

RUN apk --no-cache --update-cache --upgrade --latest add build-base git gcc bash

WORKDIR /go/src/github.com/timada-org/pikav

ADD go.mod go.mod
ADD go.sum go.sum

ENV GO111MODULE on
ENV CGO_ENABLED 1

RUN go mod download

ADD . .

RUN --mount=type=cache,target=/root/.cache/go-build go build -o /usr/bin/pikav .

FROM alpine:3.15

RUN addgroup -S timada; \
    adduser -S timada -G timada -D -u 10000 -h /home/timada -s /bin/nologin; \
    chown -R timada:timada /home/timada


COPY --from=build-env /usr/bin/pikav /usr/bin/pikav

EXPOSE 6750

USER 10000

ENTRYPOINT ["pikav"]
CMD ["serve"]
