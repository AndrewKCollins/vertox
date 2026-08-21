# Installation

VERTOX requires stable Rust and Cargo.

```bash
git clone https://github.com/AndrewKCollins/vertox.git
cd vertox
cargo install --path .
```

Check the installation:

```bash
vertox --version
vertox network
```

Optional tools:

- Foundry for `vertox build --tool foundry`
- Node.js and Hardhat for `vertox build --tool hardhat`
- Graphviz for rendering `cfg.dot`
