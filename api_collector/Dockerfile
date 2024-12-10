# 빌드 단계
FROM rust:1.83.0 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src/
RUN cargo build --release

# 실행 단계
FROM ubuntu:22.04

# 필요한 라이브러리 설치
RUN apt-get update && \
    apt-get install -y libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/api_collector /app/api_collector
CMD ["/app/api_collector"]
