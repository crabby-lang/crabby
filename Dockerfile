FROM rust:alpine

RUN apk add --no-cache \
    build-base \
    cmake \
    git

WORKDIR /usr/src/crabby
COPY . .

RUN cargo build --release

CMD ["cargo", "run", "./examples/example.crab"]
