ARG BUILDER_IMAGE=rust@sha256:9178d58b0f144a93b1dba5317d55ef32e42c67d8da71aa63ff56a4bc66f9a888

FROM ${BUILDER_IMAGE} as builder

RUN apk add --no-cache musl-dev tzdata \
        openssl-dev openssl-libs-static \
        pkgconf git libpq-dev \
        protoc protobuf-dev

# Create cobase
ENV USER=pikav
ENV UID=10001

# See https://stackoverflow.com/a/55757473/12429735
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

# Set `SYSROOT` to a dummy path (default is /usr) because pkg-config-rs *always*
# links those located in that path dynamically but we want static linking, c.f.
# https://github.com/rust-lang/pkg-config-rs/blob/54325785816695df031cef3b26b6a9a203bbc01b/src/lib.rs#L613
ENV SYSROOT=/dummy

# The env vars tell libsqlite3-sys to statically link libsqlite3.
# ENV SQLITE3_STATIC=1 SQLITE3_LIB_DIR=/usr/lib/

# The env var tells pkg-config-rs to statically link libpq.
ENV LIBPQ_STATIC=1

WORKDIR /home/pikav
COPY . /home/pikav

RUN cargo build --bin cmd --release

FROM scratch
# ARG version=unknown
# ARG release=unreleased
# LABEL name="Product Name" \
#       maintainer="info@company.com" \
#       vendor="Company AG" \
#       version=${version} \
#       release=${release} \
#       summary="High-level summary" \
#       description="A bit more details about this specific container"

COPY --from=builder /usr/share/zoneinfo /usr/share/zoneinfo
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

COPY --from=builder /home/pikav/target/release/cmd /usr/bin/pikav

USER pikav:pikav

EXPOSE 6750 6751

ENTRYPOINT [ "pikav" ]
CMD ["serve"]
