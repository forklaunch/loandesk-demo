# Builds both backend binaries (loandesk-api and loandesk-worker) into one image.
FROM rust:1.100-slim-bookworm AS build
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked \
    && cp target/release/loandesk-api target/release/loandesk-worker /usr/local/bin/

FROM debian:bookworm-slim
RUN useradd --system --uid 10001 --create-home loandesk \
    && mkdir -p /data/docs && chown loandesk:loandesk /data/docs
COPY --from=build /usr/local/bin/loandesk-api /usr/local/bin/loandesk-worker /usr/local/bin/
USER loandesk
ENV DOCS_DIR=/data/docs
EXPOSE 8080
CMD ["loandesk-api"]
