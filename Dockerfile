FROM rust:1.58 as build

RUN echo "export PATH=/usr/local/cargo/bin:$PATH" >> /etc/profile

WORKDIR /app

COPY ["./metrics/Cargo.toml", "./metrics/Cargo.lock", "./"]

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release

COPY ["./metrics/src", "./src"]

RUN cargo build --release

FROM gcr.io/distroless/cc-debian10 as final

COPY --from=build /app/target/release/metrics /

COPY ./scripts/entry.sh /

CMD ["/entry.sh"]


