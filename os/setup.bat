rustup override add stable
cargo install bootimage
rustup component add rust-src
rustup override add nightly
rustup update nightly --force
rustup component add llvm-tools-preview