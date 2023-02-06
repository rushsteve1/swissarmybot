FROM rust:alpine AS builder

RUN apk add --no-cache git musl-dev

WORKDIR /app

# Arguments for the build
ARG GIT_VERSION

# Build dependencies
RUN mkdir src && touch src/lib.rs
COPY Cargo.toml Cargo.lock .
RUN cargo build --release -j 8

# Build SAB
COPY . .
RUN cargo build --release -j 8
RUN strip /app/target/release/swiss_army_bot

# --------------- #

FROM alpine AS app

WORKDIR /app

# TODO run as nobody user
# Currently the database mount is a root and is not writeable
# RUN chown nobody:nobody /app
# USER nobody:nobody
# COPY --from=builder --chown=nobody:nobody /app/target/release/swiss_army_bot .

COPY --from=builder /app/target/release/swiss_army_bot .

VOLUME /app/swissarmy.sqlite

ENV RUST_LOG="warn"
ENV DATABASE_PATH="/app/swissarmy.sqlite"
ENV WEB_DOMAIN="sab.rushsteve1.us"

ENV STONKS_CHANNELS="859531364906172436,1026256507261689966"
ENV QOTD_CHANNELS="421467319835820035"

ENV PORT="8080"
EXPOSE 8080

CMD /app/swiss_army_bot
