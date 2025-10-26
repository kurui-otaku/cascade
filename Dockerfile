FROM rust:1.90-bookworm as builder

RUN cargo install bacon --locked

WORKDIR /workspace

# Copy entire workspace
COPY . .

WORKDIR /workspace/api

CMD ["bacon", "run"]