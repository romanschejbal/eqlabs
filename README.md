# P2P Handshake Implementation

This project provides an implementation of peer-to-peer (P2P) handshake protocols for Bitcoin, including codec and message handling functionalities. The codebase is built using Rust with the `tokio` runtime for asynchronous operations. The primary purpose of this project is to demonstrate how to establish a P2P connection with Bitcoin nodes and handle the initial handshake protocol.

## Features

- **Bitcoin Handshake**: Establish a P2P connection with a Bitcoin node and perform the initial handshake to exchange version and acknowledgment messages.
- **Message Encoding/Decoding**: Encode and decode P2P messages using custom codecs.
- **Checksum Calculation**: Calculate and verify checksums for message integrity.
- **Extendable**: The project can be extended to support other protocols (e.g., Ethereum) by implementing the necessary handshakes and message handling.

## Prerequisites

Before running the project, ensure that you have the following tools installed:

- **Rust**: Ensure you have the latest stable version of Rust installed. You can install Rust via [rustup](https://rustup.rs/).
- **Cargo**: Cargo is the Rust package manager, which is installed alongside Rust.

## How to Run

To run the Bitcoin handshake example, follow these steps:

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/yourusername/p2p-handshake.git
   cd p2p-handshake
   ```

2. **Build the Project**:
   ```bash
   cargo build --release
   ```

3. **Run the Bitcoin Handshake**:
   ```bash
   cargo run --release
   ```

   This will establish a connection with a Bitcoin node and perform the initial handshake. You should see logs indicating the connection status and message exchange.

4. **(Optional) Ethereum Handshake**:
   The Ethereum handshake is currently commented out in the `main.rs` file. You can uncomment the Ethereum-specific code and run it similarly by following the steps above.

## Code Structure

- **`src/main.rs`**: Entry point of the application, where the Bitcoin handshake is initiated.
- **`src/p2p/bitcoin.rs`**: Contains the Bitcoin handshake logic and message handling.
- **`src/codec.rs`**: Implements encoding and decoding logic for P2P messages.
- **`src/protocol.rs`**: Defines the protocol-specific commands and payload structures.
- **`src/error.rs`**: Defines error handling and custom error types.
- **`tests/`**: Contains tests for encoding, decoding, and other functionalities.

## Logging

The project uses `tracing` for (possible) structured logging. The logs provide detailed insights into the connection establishment and message exchange processes. The logging level can be adjusted by modifying the `RUST_LOG` environment variable.

```bash
RUST_LOG=info cargo run --release
```

## Contributing

Contributions are welcome! If you find any issues or have ideas for improvements, feel free to open an issue or submit a pull request.
