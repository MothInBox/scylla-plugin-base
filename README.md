# Scylla Plugin Base

WebAssembly plugins for [Scylla Reader](https://github.com/MothInBox/scylla-reader), the terminal reader that scrapes web novels through Extism plugins.

## Plugins

| Plugin | Domain | Description |
|--------|--------|-------------|
| `plugin-template` | `template` | Reference implementation. A minimal plugin that shows the full API surface. |
| `plugin-royalroad` | `royalroad` | Scraper for RoyalRoad. |

## Install a plugin

From Scylla Reader: open the command palette with `:`, choose "Install Plugin from GitHub", and enter this repo's URL. Scylla downloads the latest stable release and installs its `plugin-<domain>.wasm` assets.

To install a local build by hand, copy the `.wasm` file into `~/.config/scylla-reader/plugins/` with the name `plugin-<domain>.wasm`. The filename sets the domain, so `plugin-template.wasm` serves `template`.

> [!WARNING]
> Plugins run arbitrary wasm on your machine. Verify a plugin yourself or only use trusted sources.

## Build

Requires the `wasm32-unknown-unknown` target:

```sh
rustup target add wasm32-unknown-unknown
cargo build --release
```

Each plugin's `.cargo/config.toml` sets the wasm target and the getrandom custom backend. Built binaries land in `target/wasm32-unknown-unknown/release/`.

The template ships a Makefile that builds and installs in one step:

```sh
make -C plugin-template build
```

## Layout

- `scylla-plugin-api/`: the shared API contract. Every plugin depends on this crate.
- `plugin-*/`: one plugin per directory, each compiled to a `plugin-<domain>.wasm` binary.

## Documentation

- [docs/WRITING.md](docs/WRITING.md): write a plugin from the template, with the API types and host functions
- [docs/RELEASING.md](docs/RELEASING.md): build and publish a release that Scylla can install
