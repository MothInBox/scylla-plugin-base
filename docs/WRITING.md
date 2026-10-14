# Writing a plugin

A plugin is a Rust crate that compiles to `wasm32-unknown-unknown` and exports three functions. Start from `plugin-template/`, which implements the full surface.

## Cargo.toml

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
scylla-plugin-api = { path = "../scylla-plugin-api" }
extism-pdk = "1"
serde_json = "1"
```

Keep the path dependency pointed at this workspace's `scylla-plugin-api`. Outside the workspace, depend on the published crate instead.

## Exports

```rust
#[plugin_fn]
pub fn get_config_schema(Json(()): Json<()>) -> FnResult<Json<PluginSchema>> {
    Ok(Json(PluginSchema {
        fields: vec![
            ConfigField {
                key: "key".into(),
                label: "Display label".into(),
                field_type: "string".into(),
                default: "".into(),
            },
        ],
        accepts_cookies: false,
    }))
}

#[plugin_fn]
pub fn scrape_book(Json(input): Json<ScrapeInput>) -> FnResult<Json<ScrapeOutput>> {
    // Fetch the book page, parse it, return metadata plus the chapter list.
}

#[plugin_fn]
pub fn scrape_chapter(Json(input): Json<ScrapeInput>) -> FnResult<Json<ChapterOutput>> {
    // Fetch the chapter page and return its title and text.
}
```

## Types

The shared contract in `scylla-plugin-api`:

| Type | Fields |
|------|--------|
| `ConfigField` | `key`, `label`, `field_type`, `default` |
| `PluginSchema` | `fields: Vec<ConfigField>`, `accepts_cookies: bool` |
| `ScrapeInput` | `url: String`, `cookies: Option<String>`, `config: Option<String>` |
| `ScrapeOutput` | `title`, `url`, `cover_url: Option<String>`, `description: Option<String>`, `total_chapters: u32`, `chapters: Vec<PluginChapter>` |
| `PluginChapter` | `title`, `url`, `order: u32` |
| `ChapterOutput` | `title`, `content` |

`ScrapeInput.config` carries the user's saved settings for the plugin (see below). `accepts_cookies: true` exposes a cookie field in Scylla's plugin settings; whatever is stored there is forwarded in `ScrapeInput.cookies`.

## Host functions

Two functions come from the host:

- `curl_fetch`: download a page, with an optional cookie string.
- `scylla_fail`: abort and report an error message back to the server.

The host also permits extism-pdk's `http::request`; `plugin-royalroad` fetches that way.

## Plugin config

`get_config_schema` tells Scylla which settings to expose. Scylla writes the user's values to `~/.config/scylla-reader/plugin-<domain>.json` and passes them back in `ScrapeInput.config` on later scrapes.

## Build and install

```sh
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/my_plugin.wasm \
    ~/.config/scylla-reader/plugins/plugin-<domain>.wasm
```

The template's Makefile (`make -C plugin-template build`) does the build and copy for the `template` domain. Restart Scylla after installing by hand; releases installed from GitHub register without a restart.
