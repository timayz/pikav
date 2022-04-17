FROM gcr.io/distroless/cc

COPY ./target/release/pikav /

EXPOSE 6750

CMD ["./pikav"]
