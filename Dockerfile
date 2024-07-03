FROM rust:latest

COPY . app/

RUN apt-get update && apt-get install -y firefox-esr
RUN cargo install geckodriver

RUN cd app/ && cargo build --release && cp target/release/scout /usr/local/bin/scout
