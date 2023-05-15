FROM rust:1.67 AS builder
WORKDIR /usr/src/critiq_backend
COPY . .
RUN cargo build --release

# FROM alpine:latest  
# WORKDIR /root/
# COPY --from=builder /usr/src/critiq_backend/target/release/critiq_backend ./
CMD ["./target/release/critiq_backend"]