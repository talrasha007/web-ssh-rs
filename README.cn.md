# web-ssh-rs

一个用 Rust 实现的、类似 [wetty](https://github.com/butlerx/wetty) 的极简 Web SSH 终端。

它启动一个 HTTP 服务，让浏览器通过 [xterm.js](https://xtermjs.org/) 和 WebSocket 打开一个交互式
SSH shell。仅此而已——没有用户账号体系、没有 TLS、没有会话存储，这些都交给前面的反向代理/网关去做。

English version: [README.md](README.md)。

## 特性

- 通过 URL 连接任意 SSH 主机：路径形式（`/ssh/user@host` 或 `/ssh/user@host:port`）或查询参数
  形式（`/ssh?user=&host=` 或 `/ssh?user=&host=&port=`）。不传 `port` 时默认使用 `22`。
- 两种认证方式：
  - **公钥认证** —— 启动时通过 `-i`/`--identity-file` 配置一把私钥，用它以 URL 中指定的用户名去
    认证目标主机（类似跳板机模式）。
  - **交互式密码认证** —— 不传 `-i` 时，浏览器终端里会像真实的 `ssh` 客户端一样提示输入密码（不
    回显、支持退格纠错、认证失败有重试次数限制）。
- 终端使用 `xterm.js` 渲染（通过 CDN 加载，前端无需任何构建步骤）。
- 单一自包含二进制文件——前端 HTML/JS 在编译期直接嵌入可执行文件。
- 不做 HTTP 鉴权、不做 TLS。请把它部署在能处理这两者的网关后面。

## 快速开始

```sh
cargo build --release

# 公钥模式：用这把私钥以 URL 中的用户名认证任意目标主机
./target/release/web-ssh-rs -i ~/.ssh/id_ed25519

# 密码模式：改为在浏览器终端里交互式输入密码
./target/release/web-ssh-rs
```

然后打开：

```
http://localhost:8080/ssh/root@203.0.113.10:22
http://localhost:8080/ssh?user=root&host=203.0.113.10&port=22
```

### 命令行参数

| 参数 | 默认值 | 说明 |
|---|---|---|
| `-i, --identity-file <PATH>` | （无） | 用于公钥认证的 SSH 私钥路径。不传则回退为交互式密码认证。 |
| `--key-passphrase <PASSPHRASE>` | （无） | 私钥文件的密码（如果私钥被加密）。 |
| `--bind-addr <ADDR>` | `0.0.0.0:8080` | HTTP 服务监听地址。 |

## Docker

```sh
docker build -t web-ssh-rs .

docker run -d -p 8080:8080 \
  -v /path/to/id_ed25519:/keys/id_ed25519:ro \
  web-ssh-rs -i /keys/id_ed25519
```

每次发布 GitHub Release 时，会自动构建并推送多架构（`amd64`/`arm64`）镜像到 GHCR：

```sh
docker pull ghcr.io/talrasha007/web-ssh-rs:latest
```

运行时镜像基于 `gcr.io/distroless/cc-debian12`（无 shell、无包管理器、以非 root 用户运行），
以尽量缩小操作系统层面的攻击面。

## Release 流程

Release 发布时会触发两个相互独立的 GitHub Actions workflow：

- **[docker-release.yml](.github/workflows/docker-release.yml)** —— 构建并推送
  `linux/amd64` + `linux/arm64` 镜像到 GHCR，同时打上 `latest` 和 release 版本号两个 tag。
- **[binary-release.yml](.github/workflows/binary-release.yml)** —— 为
  `x86_64`/`aarch64` Linux 和 `x86_64`/`aarch64` macOS 构建独立二进制文件，并上传到 Release 资源中。

## 架构

```
浏览器 (xterm.js)  <-- WebSocket -->  web-ssh-rs (axum)  <-- SSH (russh) -->  目标 sshd
```

- HTTP/WebSocket 服务：[axum](https://github.com/tokio-rs/axum)
- SSH 客户端：[russh](https://github.com/Eugeny/russh)（纯 Rust 实现，不依赖系统 `ssh` 命令）
- 前端：静态 `xterm.js` 页面，无需任何构建工具链

浏览器 → 服务端的消息是小体积的 JSON 文本帧（`{"type":"data",...}` /
`{"type":"resize",...}`）；服务端 → 浏览器的终端输出则以原始二进制 WebSocket 帧发送。

## 安全说明

本项目有意只实现最核心的代理功能：

- SSH 主机密钥校验**已关闭**（接受任意主机密钥），没有 known_hosts 存储。
- HTTP/WebSocket 层没有任何鉴权，也没有 TLS。

请务必将它部署在能够限制访问、并终结 TLS 的网关/反向代理之后再使用。

## License

Apache License 2.0，详见 [LICENSE](LICENSE)。
