FROM rust:1.80.1 as builder

RUN apt-get update && apt-get install -y pkg-config dav1d libdav1d-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/yliproxy
COPY . .

RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ffmpeg && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/yliproxy /usr/local/bin/yliproxy
CMD ["yliproxy"]