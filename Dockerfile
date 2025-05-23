ARG APP_NAME="patreon-reader-service"
FROM rust:1.86 AS builder
ARG APP_NAME

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
ARG APP_NAME
WORKDIR /app
COPY --from=builder /app/target/release/$APP_NAME /usr/local/bin/$APP_NAME
CMD [ $APP_NAME ]

