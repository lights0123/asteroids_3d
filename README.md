# asteroids_3d

# Building for Native

```bash
cargo build --release
```

# Web

## Quick test mode

```bash
wasm-pack build -t no-modules -- --features uncached-web-assets # optionally --dev
```

Then, serve the root directory, e.g. `basic-http-server`.

## Full version

```bash
wasm-pack build -t no-modules
cd web
yarn
```

To run the dev server, run `yarn dev`. To build, run `yarn build`.
