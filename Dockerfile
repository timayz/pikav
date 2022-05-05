FROM gcr.io/distroless/static

COPY ./pikav /

EXPOSE 6750

CMD ["/pikav"]
