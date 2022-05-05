FROM gcr.io/distroless/static

COPY ./pikav /

EXPOSE 6750

CMD ["/pikav"]

# RUN CGO_ENABLED=0 go build -o /go/bin/app