#Generates lock file from dependencies
FROM lukemathwalker/cargo-chef:latest-rust-1.53.0 as planner
WORKDIR /app

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

#Instlal dependencies to /usr/local/cargo based on lock file
FROM lukemathwalker/cargo-chef:latest-rust-1.53.0 as cacher
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

#Builds release applicaiton, copying usr/local from cacher
FROM rust:1.53.0 AS builder
WORKDIR /app

COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --release

FROM debian:buster-slim as runtime
WORKDIR /app

RUN apt-get update -y \
	&& apt-get install -y --no-install-recommends openssl \
	# Clean up
	&& apt-get autoremove -y \
	&& apt-get clean -y \
	&& rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production

ENTRYPOINT ["./zero2prod"]