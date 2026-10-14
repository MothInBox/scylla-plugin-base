# Releasing a plugin

Users install plugins from a repo's GitHub releases, so the release has to carry correctly named `.wasm` assets.

## Build

```sh
cargo build --target wasm32-unknown-unknown --release
```

## Name the asset

Every asset must be named `plugin-<domain>.wasm`. Scylla takes the domain from the filename: `plugin-example.com.wasm` registers for `example.com`. An asset named any other way gets installed but not registered to a domain.

## Publish

Cargo emits the wasm under the package name with underscores, so copy it to the name Scylla expects before publishing:

```sh
cp target/wasm32-unknown-unknown/release/my_plugin.wasm plugin-<domain>.wasm
gh release create v0.1.0 plugin-<domain>.wasm
```

Scylla fetches the **latest stable release** and downloads every `.wasm` asset attached to it, whatever the directory structure. A single release can bundle several plugins, and each domain registers independently.

## Verify the install

In Scylla Reader, open the command palette (`:`), run "Install Plugin from GitHub" with the repo URL, then add a book from the plugin's domain and confirm the chapters scrape.
