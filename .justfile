# Derive the binary name from the current directory
CRATE_NAME := `basename $(pwd)`


# Install root for `cargo install`
INSTALL_ROOT := "$HOME/.local"

# Default task
default: upgrade

# Run the unit‐tests
test:
    cargo test

# Build + install without overwriting
install:
    just test
    cargo install --path . --locked --root {{INSTALL_ROOT}}

# Build + install, overwriting any existing binary
upgrade:
    just test
    cargo install --path . --root {{INSTALL_ROOT}} --force


