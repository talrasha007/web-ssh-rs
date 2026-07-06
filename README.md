# web-ssh-rs

A minimal, [wetty](https://github.com/butlerx/wetty)-style web SSH terminal, written in Rust.

It runs an HTTP server that lets a browser open an interactive SSH shell through
[xterm.js](https://xtermjs.org/) over a WebSocket. Nothing more — no user accounts, no TLS, no
session store. Those are expected to be handled by whatever reverse proxy/gateway sits in front
of it.

中文说明见 [README.cn.md](README.cn.md)。

## Features

- Connect to any SSH host by URL: path style (`/ssh/user@host` or `/ssh/user@host:port`) or
  query style (`/ssh?user=&host=` or `/ssh?user=&host=&port=`). `port` defaults to `22` when
  omitted.
- Two authentication modes:
  - **Public key** — configure one private key at startup (`-i`/`--identity-file`); it's used to
    authenticate as whatever user the URL specifies (bastion/jump-box style).
  - **Interactive password** — omit `-i` and the browser terminal prompts for a password just
    like a normal `ssh` client (no echo, backspace support, limited retries).
- Terminal rendered with `xterm.js` (loaded from a CDN, no frontend build step).
- Single self-contained binary — the HTML/JS frontend is embedded into the executable at compile
  time.
- No HTTP auth, no TLS. Put this behind a gateway that handles both.

## Quick start

```sh
cargo build --release

# Public-key mode: authenticate to every requested host as its URL user, using this key
./target/release/web-ssh-rs -i ~/.ssh/id_ed25519

# Password mode: prompt for a password in the browser terminal instead
./target/release/web-ssh-rs
```

Then open:

```
http://localhost:8080/ssh/root@203.0.113.10:22
http://localhost:8080/ssh?user=root&host=203.0.113.10&port=22
```

### CLI options

| Flag | Default | Description |
|---|---|---|
| `-i, --identity-file <PATH>` | _(none)_ | SSH private key used for public-key auth. Omit to fall back to interactive password auth. |
| `--key-passphrase <PASSPHRASE>` | _(none)_ | Passphrase for an encrypted identity file. |
| `--bind-addr <ADDR>` | `0.0.0.0:8080` | Address the HTTP server listens on. |

## Docker

```sh
docker build -t web-ssh-rs .

docker run -d -p 8080:8080 \
  -v /path/to/id_ed25519:/keys/id_ed25519:ro \
  web-ssh-rs -i /keys/id_ed25519
```

Prebuilt images are published to GHCR on every GitHub release:

```sh
docker pull ghcr.io/talrasha007/web-ssh-rs:latest
```

The runtime image is based on `gcr.io/distroless/cc-debian12` (no shell, no package manager, runs
as non-root) to keep the OS-level attack surface minimal.

## Releases

Two independent GitHub Actions workflows run when a release is published:

- **[docker-release.yml](.github/workflows/docker-release.yml)** — builds and pushes an image to
  GHCR, tagged `latest` and the release tag. By default it only builds `linux/amd64`, since
  `linux/arm64` has to be emulated via QEMU on the `ubuntu-latest` runner and is much slower to
  build. To also build `linux/arm64` for a given release, run this workflow manually
  (`workflow_dispatch`) with the release tag and `include_arm64: true`.
- **[binary-release.yml](.github/workflows/binary-release.yml)** — builds standalone binaries for
  `x86_64`/`aarch64` Linux and `x86_64`/`aarch64` macOS, and uploads them as release assets.

## Architecture

```
Browser (xterm.js)  <-- WebSocket -->  web-ssh-rs (axum)  <-- SSH (russh) -->  target sshd
```

- HTTP/WebSocket server: [axum](https://github.com/tokio-rs/axum)
- SSH client: [russh](https://github.com/Eugeny/russh) (pure Rust, no system `ssh` binary
  dependency)
- Frontend: static `xterm.js` page, no build tooling

Browser → server messages are small JSON text frames (`{"type":"data",...}` /
`{"type":"resize",...}`); server → browser terminal output is sent as raw binary WebSocket frames.

## Security notes

This project intentionally implements only the core proxy functionality:

- SSH host key verification is **disabled** (accepts any host key). There's no known_hosts store.
- No authentication or TLS on the HTTP/WebSocket layer.

Run it only behind a gateway/reverse proxy that restricts access and terminates TLS.

## License

Apache License 2.0 — see [LICENSE](LICENSE).
