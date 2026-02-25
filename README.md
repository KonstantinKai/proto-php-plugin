# proto-php-plugin

[![CI](https://github.com/KonstantinKai/proto-php-plugin/actions/workflows/ci.yml/badge.svg)](https://github.com/KonstantinKai/proto-php-plugin/actions/workflows/ci.yml)
[![Release](https://github.com/KonstantinKai/proto-php-plugin/actions/workflows/release.yml/badge.svg)](https://github.com/KonstantinKai/proto-php-plugin/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A community [WASM plugin](https://moonrepo.dev/docs/proto/wasm-plugin) for [proto](https://github.com/moonrepo/proto) that manages PHP versions using pre-built static binaries.

## Installation

```bash
proto plugin add php "github://KonstantinKai/proto-php-plugin"
proto install php
```

## Usage

```bash
# Install PHP
proto install php 8.5

# Use PHP
proto run php -- --version

# List available versions
proto versions php

# Pin a version in the current directory
proto pin php 8.5
```

## Version Detection

The plugin automatically detects PHP versions from:

- `.php-version` — plain version string (e.g. `8.5.3`)
- `composer.json` — reads `require.php` field (supports `>=`, `^`, `~`, `||` constraints)

## Configuration

Configure in `.prototools` under `[tools.php]`:

```toml
[tools.php]
# Disable prebuilt binaries and build from source (default: true)
prebuilt = false

# SAPI type: "cli", "fpm", or "micro" (default: "cli")
# Applies to prebuilt binaries and spc source builds (Linux/macOS only)
sapi = "fpm"

# Custom download URL template (advanced)
# Placeholders: {version}, {sapi}, {os}, {arch}, {file}
dist-url = "https://example.com/php-{version}-{sapi}-{os}-{arch}.tar.gz"

# Extra flags passed to ./configure when building from source (php.net path)
# NOTE: Setting this forces the traditional configure/make build even for PHP 8.x
configure-opts = ["--with-pdo-pgsql", "--enable-intl", "--enable-soap"]

# Extensions for spc (static-php-cli) source builds (PHP 8.x only)
# Defaults to the 37 extensions matching prebuilt binaries
extensions = ["bcmath", "curl", "mbstring", "openssl", "pdo", "pdo_sqlite", "zip"]
```

## SAPI Selection

The [static-php-cli](https://static-php.dev) project provides prebuilt binaries for three SAPI types:

| SAPI    | Description                                         |
|---------|-----------------------------------------------------|
| `cli`   | Standard command-line interface (default)            |
| `fpm`   | FastCGI Process Manager for web serving              |
| `micro` | Self-contained micro binary (single-file executable) |

```toml
[tools.php]
sapi = "fpm"
```

The `sapi` option applies to Linux and macOS prebuilt binaries and spc source builds. Windows binaries from `windows.php.net` are always NTS CLI builds and ignore this setting. The php.net source build path also ignores this setting — use `configure-opts` with `--enable-fpm` instead.

## Supported Platforms

| Platform       | Architecture      | Source                                  |
|----------------|-------------------|-----------------------------------------|
| Linux          | x64, arm64        | [static-php-cli](https://static-php.dev) prebuilt |
| macOS          | x64, arm64        | [static-php-cli](https://static-php.dev) prebuilt |
| Windows        | x64               | [windows.php.net](https://windows.php.net) official |

## Building from Source

If a prebuilt binary isn't available for your PHP version, proto will automatically fall back to building from source. Set `prebuilt = false` to always build from source.

Building from source is not supported on Windows.

### Build methods

The plugin uses two build methods depending on the PHP version and configuration:

| Condition | Build method | Result |
|---|---|---|
| PHP 8.x, no `configure-opts` | spc ([static-php-cli](https://static-php.dev)) | Static binary, zero dependencies |
| PHP 8.x, `configure-opts` set | php.net (`configure`/`make`) | Traditional build with custom flags |
| PHP 7.x | php.net (`configure`/`make`) | Traditional build |

### spc builds (PHP 8.x default)

For PHP 8.x without custom configure flags, the plugin uses [static-php-cli](https://static-php.dev) (`spc`) to produce fully-static binaries — the same quality as prebuilt downloads. The spc binary is downloaded automatically.

```
curl -fsSL -o spc https://dl.static-php.dev/static-php-cli/spc-bin/nightly/spc-{os}-{arch}
./spc download --with-php={version} --for-extensions={exts} --prefer-pre-built
./spc build {exts} --build-{sapi}
```

By default, spc builds include the same 37 extensions as the prebuilt binaries. You can customize the extension list:

```toml
[tools.php]
prebuilt = false
extensions = ["bcmath", "curl", "mbstring", "openssl", "pdo", "pdo_sqlite", "zip"]
```

Proto will prompt to install missing system packages automatically (via `apt`, `brew`, or `dnf`).

**System dependencies (spc):**

| apt | brew | dnf |
|---|---|---|
| `build-essential`, `cmake`, `pkg-config` | `cmake`, `pkg-config` | `gcc`, `gcc-c++`, `make`, `cmake`, `pkgconf` |

### php.net builds (PHP 7.x / custom configure flags)

For PHP 7.x, or when `configure-opts` is set, the plugin downloads the source tarball from `php.net/distributions/` and runs:

```
./configure --prefix=<install-dir> --enable-mbstring --enable-bcmath \
  --enable-pcntl --with-openssl --with-curl --with-zlib --with-pdo-sqlite
make -j4
make install
```

NOTE: Setting `configure-opts` forces the php.net build path even for PHP 8.x.

**System dependencies (php.net):**

**Ubuntu/Debian (apt):**

```
build-essential autoconf bison re2c pkg-config libxml2-dev libsqlite3-dev
libssl-dev libcurl4-openssl-dev libonig-dev libreadline-dev libzip-dev
zlib1g-dev libicu-dev
```

**macOS (brew):**

```
autoconf bison re2c pkg-config libxml2 openssl@3 curl oniguruma readline
libzip zlib icu4c
```

**Fedora/RHEL (dnf):**

```
gcc gcc-c++ make autoconf bison re2c pkgconf libxml2-devel sqlite-devel
openssl-devel libcurl-devel oniguruma-devel readline-devel libzip-devel
zlib-devel libicu-devel
```

### Custom configure flags

Add extra `./configure` flags via `configure-opts`. These are appended after the defaults and force the php.net build path:

```toml
[tools.php]
prebuilt = false
configure-opts = [
  "--enable-intl",
  "--enable-soap",
  "--enable-fpm",
  "--with-pdo-pgsql",
  "--with-pdo-mysql",
  "--with-readline",
  "--with-zip",
]
```

### macOS: Xcode SDK libxml2 issue

On macOS with Xcode 16+ (Sequoia), PHP's dom extension fails to compile against the system SDK `libxml2` headers. To fix this, point `./configure` to Homebrew's `libxml2`:

```toml
[tools.php]
prebuilt = false
configure-opts = ["--with-libxml=/opt/homebrew/opt/libxml2"]
```

On Intel Macs, use `/usr/local/opt/libxml2` instead.

### macOS: PHP 7.x OpenSSL 3.x incompatibility

PHP 7.x uses deprecated OpenSSL 1.x APIs that fail to compile against OpenSSL 3.x (shipped by default on modern macOS). If `make` fails with hundreds of `deprecated` warnings in `ext/openssl/openssl.c`, either point to OpenSSL 1.1:

```toml
[tools.php]
prebuilt = false
configure-opts = ["--with-openssl=/opt/homebrew/opt/openssl@1.1"]
```

Requires `brew install openssl@1.1`. On Intel Macs, use `/usr/local/opt/openssl@1.1` instead.

Or build without OpenSSL:

```toml
[tools.php]
prebuilt = false
configure-opts = ["--without-openssl"]
```

### macOS: PHP 7.x DNS resolver linking error

If linking fails with `Undefined symbols for architecture arm64: "_res_9_dn_expand"`, PHP 7.x needs the resolver library explicitly linked on modern macOS:

```toml
[tools.php]
prebuilt = false
configure-opts = ["EXTRA_LIBS=-lresolv"]
```

### macOS: PHP 7.x PCRE JIT allocation failure

If `make` fails with `Allocation of JIT memory failed` while generating `phar.phar`, PCRE JIT can't allocate executable memory on macOS arm64 due to security restrictions. Disable it at compile time:

```toml
[tools.php]
prebuilt = false
configure-opts = ["--without-pcre-jit"]
```

### macOS: iconv issue

If `./configure` fails with `Please specify the install prefix of iconv`, either install iconv via Homebrew and point to it, or disable it:

```toml
[tools.php]
prebuilt = false
configure-opts = ["--without-iconv"]
```

### spc: GitHub API rate limit

The `spc download` command fetches release info from the GitHub API, which limits unauthenticated requests to 60/hour per IP. If you see a 403 error like:

```
curl: (56) The requested URL returned error: 403
Download failed: Failed to fetch "https://api.github.com/repos/static-php/static-php-cli-hosted/releases"
```

Set a GitHub token before installing:

```bash
export GITHUB_TOKEN="ghp_your_token_here"
proto install php 8.5
```

Generate a token at **GitHub → Settings → Developer settings → Personal access tokens → Fine-grained tokens** — no special permissions needed.

## PHP Extensions

### Prebuilt binaries (default)

The static-php-cli prebuilt binaries ship with 37 extensions compiled in ([source](https://dl.static-php.dev/static-php-cli/common/README.txt)):

`bcmath`, `bz2`, `calendar`, `ctype`, `curl`, `dom`, `exif`, `fileinfo`, `filter`, `ftp`, `gd`, `gmp`, `iconv`, `mbregex`, `mbstring`, `mysqlnd`, `openssl`, `pcntl`, `pdo`, `pdo_mysql`, `pdo_pgsql`, `pdo_sqlite`, `pgsql`, `phar`, `posix`, `redis`, `session`, `simplexml`, `soap`, `sockets`, `sqlite3`, `tokenizer`, `xml`, `xmlreader`, `xmlwriter`, `zip`, `zlib`

These extensions are statically linked and always available — no additional installation needed. However, `pecl`, `phpize`, and `php-config` are **not included** in prebuilt binaries, so installing additional extensions via PECL is not possible.

### Source builds

When building from source (`prebuilt = false`), `pecl`, `phpize`, and `php-config` are installed alongside PHP. You can install additional extensions via PECL:

```bash
pecl install redis
pecl install xdebug
```

Add extensions at compile time using `configure-opts`:

```toml
[tools.php]
prebuilt = false
configure-opts = ["--enable-intl", "--enable-soap", "--with-pdo-pgsql"]
```

## Prebuilt Binary Availability

The static-php-cli project provides prebuilt binaries for PHP 8.x, but coverage is sporadic for some minor versions (especially 8.2.x and 8.3.x). PHP 8.4.x has the best coverage.

If a prebuilt binary is not available for your requested version, `proto install` will fail with a download error. In that case:

- Try a nearby patch version (e.g. `8.3.20` instead of `8.3.15`)
- Or set `prebuilt = false` to build from source

Windows prebuilt binaries come from the official `windows.php.net` and cover all released versions.

## Version Aliases

- `latest` — resolves to the newest stable PHP release
- `stable` / `lts` — same as `latest`
- Partial versions like `8.5` resolve to the latest patch (e.g. `8.5.3`)

## Comparison with phpenv / asdf-php

| Feature | phpenv | asdf-php | proto-php-plugin |
|---|---|---|---|
| **Install method** | Compile from source | Compile from source | Prebuilt binaries |
| **Install time** | 5–15 min | 5–15 min | Seconds |
| **System deps needed** | Many (compiler, libs) | Many (compiler, libs) | None |
| **Linux** | x64 | x64 | x64, arm64 |
| **macOS** | x64, arm64 | x64, arm64 | x64, arm64 |
| **Windows** | No | No | x64 |
| **Version file** | `.php-version` | `.tool-versions` | `.php-version`, `composer.json` |
| **Build from source** | Only option | Only option | Opt-in fallback |
| **Bundled extensions** | User-configured | User-configured | 37 built-in |

## Related

- [proto-composer-plugin](https://github.com/KonstantinKai/proto-composer-plugin) — Composer (PHP dependency manager) version management for proto

## License

MIT
