FROM rust:1.56 as build

WORKDIR /senseid

# copy your source tree
COPY . .

RUN rustup component add rustfmt

RUN cargo build --verbose --release

# our final base
FROM debian:buster-slim

# copy the build artifact from the build stage
COPY --from=build /senseid/target/release/senseid .

# set the startup command to run your binary
CMD ["./senseid"]