# syntax=docker/dockerfile:1

FROM --platform=$BUILDPLATFORM rust:1.69 as build
ARG TARGETARCH

RUN echo "export PATH=/usr/local/cargo/bin:$PATH" >> /etc/profile
WORKDIR /app

COPY ["./build/platform.sh", "./"]
RUN ./platform.sh
COPY ["./build/.cargo/config", ".cargo/config"]
RUN rustup target add $(cat /.platform)
RUN apt-get update && apt-get install -y $(cat /.compiler)

COPY ["./metrics/Cargo.toml", "./metrics/Cargo.lock", "./"]

RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release --target=$(cat /.platform)

COPY ["./metrics/src", "./src"]

RUN touch src/main.rs && cargo build --release --target=$(cat /.platform)

RUN mkdir -p /release/$TARGETARCH
RUN cp ./target/$(cat /.platform)/release/metrics /release/$TARGETARCH/metrics

FROM gcr.io/distroless/cc-debian11
ARG TARGETARCH
COPY --from=build /release/$TARGETARCH/metrics /

CMD ["/metrics"]