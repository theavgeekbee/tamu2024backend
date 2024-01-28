# Financy Backend
Financy is a project compiled by Adriana Zambrano and Nathan Lee for
TAMU Hack 2024. It is a budget tracker and financial literacy hub for information.
This repository contains the backend for this project, written in Rust.
## Requirements
This was written in
- `rustc` 1.74.1
- Cargo 1.74.1
- RustRover EAP 2023.3\

RustRover is not required, but it is required that you have a `rustc` or Cargo version greater than or equal to what is listed here
## Building
1. Download dependents using `cargo c`
2. Then, run `cargo run` to start the server in the development environment.
## Deployment
1. Download dependents using `cargo c`
2. From here, you can do one of two things.
   1. Run `cargo run --release` to start the server in the production environment.
   2. You can also run `cargo build --release` to build the binary, and then run it from the `target/release` directory.
3. However, it is required that the `Rocket.toml` file is in the working directory.