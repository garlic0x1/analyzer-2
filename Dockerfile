FROM rust:latest

WORKDIR /usr/src/analyzer
COPY . .

RUN cargo install --path .

CMD ["analyzer"]