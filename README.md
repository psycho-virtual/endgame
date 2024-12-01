# EndGame

A minimal implementation of a succinct blockchain using Reed-Solomon accumulation schemes and density-based consensus.

## Overview

This project demonstrates core concepts of a next-generation blockchain architecture focusing on three main goals:
- Fast finality through density-based consensus
- Succinctness via Reed-Solomon accumulation schemes
- Efficient parallel proof generation

## Key Components

- `accumulator/`: Reed-Solomon accumulation scheme implementation
- `consensus/`: Density-based fork choice rules
- `crypto/`: Field arithmetic and cryptographic primitives

## Getting Started

### Prerequisites

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building

```bash
git clone https://github.com/yourusername/endgame.git
cd endgame
cargo build
```

### Running Tests

```bash
cargo test
```

For verbose test output:
```bash
cargo test -- --show-output
```

To run specific tests:
```bash
cargo test test_density_consensus -- --exact --show-output
```

## Project Structure

```
EndGame/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── accumulator/
│   │   ├── mod.rs          # Accumulator trait definitions
│   │   └── reed_solomon.rs # RS-based accumulation implementation
│   ├── blockchain/
│   │   ├── mod.rs          # Core blockchain types
│   │   ├── block.rs        # Block and transaction structures  
│   │   └── state.rs        # State management
│   ├── consensus/
│   │   ├── mod.rs          # Consensus trait definitions
│   │   └── density.rs      # Density-based fork choice rules
│   ├── crypto/
│   │   ├── mod.rs          # Cryptographic primitives
│   │   ├── field.rs        # Finite field operations
│   │   └── merkle.rs       # Simple Merkle tree implementation
│   └── network/
│       ├── mod.rs          # Basic networking interfaces
│       └── node.rs         # Node implementation
└── tests/
    └── integration_tests.rs
```

## Core Features

1. **Reed-Solomon Accumulation**
   - Constant-size proofs regardless of chain length
   - Efficient verification
   - Support for proof folding

2. **Density-Based Consensus**
   - Window-based density calculations
   - Efficient fork choice rules
   - Time-slot based block validation

3. **Field Operations**
   - Implementation over a Mersenne prime field
   - Support for basic finite field arithmetic

## Development Status

This is a minimal implementation focused on demonstrating core concepts. Many features you'd want in a production system are intentionally omitted, such as:
- Network layer
- Transaction types beyond basic state transitions
- Advanced consensus optimizations
- Full node/client interfaces

## Dependencies

```toml
[dependencies]
rand = "0.8"          # For cryptographic randomness
sha2 = "0.10"         # For hash functions
rayon = "1.7"         # For parallelization
```

## License

MIT License

Copyright (c) 2024 EndGame Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
