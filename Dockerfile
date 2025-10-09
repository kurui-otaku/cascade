FROM rust:1.86-bookworm as builder

RUN cargo install bacon --locked

WORKDIR /workspace

# Copy entire workspace
COPY . .

WORKDIR /workspace/api

CMD ["bacon", "run"]