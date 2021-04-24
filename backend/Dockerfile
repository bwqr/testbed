FROM rust:1 as builder

WORKDIR /usr/src/kr-testbed-api

COPY . .

RUN cargo build -p api --release

FROM debian:buster-slim

RUN apt-get update && apt-get install -y libpq5 libssl1.1

WORKDIR /usr/local/apps/nrgtestbed

COPY --from=builder /usr/src/kr-testbed-api/target/release/api .

EXPOSE 8080

CMD ["./api"]
