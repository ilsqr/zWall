#!/bin/bash

# Rust nightly gerekli
echo "Setting up Rust nightly..."
rustup toolchain install nightly
rustup default nightly

# eBPF target ekleme
echo "Adding eBPF targets..."
rustup target add bpfel-unknown-none
rustup target add bpfeb-unknown-none

# bpf-linker kurma
echo "Installing bpf-linker..."
cargo install bpf-linker

# eBPF programını derleme
echo "Building eBPF program..."
cd ping-blocker-ebpf
cargo build --release --target bpfel-unknown-none
cd ..

# Kullanıcı alanı programını derleme
echo "Building user space program..."
cd ping-blocker
cargo build --release
cd ..

echo "Build completed!"
echo "Run with: sudo ./target/release/ping-blocker --iface <interface> --threshold <number>"
