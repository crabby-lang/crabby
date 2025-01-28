FROM rust:alpine

RUN apk add --no-cache \
    build-base \
    cmake \
    git

WORKDIR /usr/src/crabby
COPY . .

RUN cargo build

CMD ["cargo", "run", "./examples/example.crab"]
