# Search the live mem0 memory with a sample query (needs MEM0_LITELLM_KEY).
dev query="commit conventions":
    cargo run -q -p mem0-recall -- search {{query}}

test:
    cargo test --workspace

lint:
    cargo clippy --all-targets -- -D warnings

fmt:
    cargo fmt --all

check:
    cargo fmt --all --check
    just lint
    just test

install:
    cargo install --path crates/mem0-recall

# Install the mem0 skill for fennec (global skills directory).
install-fennec:
    mkdir -p ~/.agents/skills
    cp -r adapters/fennec/skills/mem0 ~/.agents/skills/
