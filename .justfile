# Derive the binary name from the current directory
CRATE_NAME := `basename $(pwd)`


# Install root for `cargo install`
INSTALL_ROOT := "$HOME/.local"

# Default task
default: upgrade


# Build + install without overwriting
install:
    cargo install --path . --locked --root {{INSTALL_ROOT}}

# Build + install, overwriting any existing binary
upgrade:
    cargo install --path . --locked --root {{INSTALL_ROOT}} --force


