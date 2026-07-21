# anti-spam-matrix

A simple Matrix spam moderation bot.

The current detection logic is intentionally straightforward:

* If a user sends a specified keyword in a configured number of **consecutive messages**, they will be banned.
* If a user triggers the keyword in **consecutive messages across multiple rooms** (reaching the configured `spam_limit`), they will also be banned.

Once a user is identified as a spammer, the bot will ban them from **every room where it has sufficient permissions**.

## Building

To create a standard release build:

```bash
cargo build --release
```

To create a statically linked build:

```bash
cargo build --release --no-default-features \
    -F eyra-as-std \
    -F rustls-tls \
    -F socks
```

## Configuration

### Authentication

The bot currently supports two authentication methods: `password` and `sso_login`.

**Password authentication:**

```toml
[auth]
type = "password"
password = "VeryHardPassword"
```

> [!NOTE]
> When using SSO authentication, the username portion of the Matrix user ID is ignored.

**SSO authentication:**

```toml
[auth]
type = "sso_login"
```

### Proxy

SOCKS5 proxy:

```toml
proxy = "socks5://114.51.41.191:9810"
```

HTTP proxy with authentication:

```toml
proxy = "http://name:passwd@114.51.41.191:9810"
```
