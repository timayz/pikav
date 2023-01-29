FROM gcr.io/distroless/cc

COPY ./target/release/cmd /usr/bin/pikav

EXPOSE 6750

ENTRYPOINT ["pikav"]
CMD ["serve"]
