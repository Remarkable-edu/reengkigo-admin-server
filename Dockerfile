# syntax=docker/dockerfile:1
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin migrate_data 
RUN cargo build --release --bin reengkigo-admin-app 

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/reengkigo-admin-app /usr/local/bin/reengkigo-admin-app
COPY --from=builder /app/target/release/migrate_data /usr/local/bin/migrate_data
COPY static ./static
COPY asset ./asset
RUN chmod -R 777 ./asset
COPY src/templates ./src/templates
COPY project_list.yaml .
EXPOSE 3000
ENV APP_SERVER_HOST=0.0.0.0
ENV APP_SERVER_PORT=3000
CMD ["reengkigo-admin-app"]