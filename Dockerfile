FROM gcr.io/distroless/cc

COPY ./target/release/pikav-cli /usr/bin/pikav

EXPOSE 6750

ENTRYPOINT ["pikav"]
CMD ["serve"]
