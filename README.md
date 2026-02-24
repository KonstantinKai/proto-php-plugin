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
proto install php 8.4

# Use PHP
proto run php -- --version

# List available versions
proto versions php

# Pin a version in the current directory
proto pin php 8.4
```

## Version Detection

The plugin automatically detects PHP versions from:

- `.php-version` — plain version string (e.g. `8.4.12`)
- `composer.json` — reads `require.php` field (supports `>=`, `^`, `~`, `||` constraints)

## Configuration

Configure in `.prototools` under `[tools.php]`:

```toml
[tools.php]
# Disable prebuilt binaries and build from source (default: true)
prebuilt = false

# Custom download URL template (advanced)
# Placeholders: {version}, {os}, {arch}, {file}
dist-url = "https://example.com/php-{version}-cli-{os}-{arch}.tar.gz"

# Extra flags passed to ./configure when building from source
configure-opts = ["--with-pdo-pgsql", "--enable-intl", "--enable-soap"]
```

## Supported Platforms

| Platform       | Architecture      | Source                                  |
|----------------|-------------------|-----------------------------------------|
| Linux          | x64, arm64        | [static-php-cli](https://static-php.dev) prebuilt |
| macOS          | x64, arm64        | [static-php-cli](https://static-php.dev) prebuilt |
| Windows        | x64               | [windows.php.net](https://windows.php.net) official |

## Building from Source

If a prebuilt binary isn't available for your PHP version, proto will automatically fall back to building from source using official `php.net` tarballs. Set `prebuilt = false` to always build from source.

Building from source is not supported on Windows.

### How it works

The plugin downloads the source tarball from `php.net/distributions/php-{version}.tar.xz`, then runs:

```
./configure --prefix=<install-dir> --enable-mbstring --enable-bcmath \
  --enable-pcntl --with-openssl --with-curl --with-zlib --with-pdo-sqlite
make -j4
make install
```

Proto will prompt to install missing system packages automatically (via `apt`, `brew`, or `dnf`).

### System dependencies

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

Add extra `./configure` flags via `configure-opts`. These are appended after the defaults:

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

### macOS: iconv issue

If `./configure` fails with `Please specify the install prefix of iconv`, either install iconv via Homebrew and point to it, or disable it:

```toml
[tools.php]
prebuilt = false
configure-opts = ["--without-iconv"]
```

## PHP Extensions

### Prebuilt binaries (default)

The static-php-cli prebuilt binaries ship with ~36 extensions compiled in. Key extensions include:

`bcmath`, `calendar`, `ctype`, `curl`, `dom`, `exif`, `fileinfo`, `filter`, `ftp`, `gd`, `iconv`, `json`, `mbstring`, `mysqli`, `mysqlnd`, `openssl`, `opcache`, `pcntl`, `pdo`, `pdo_mysql`, `pdo_pgsql`, `pdo_sqlite`, `phar`, `posix`, `readline`, `session`, `simplexml`, `sockets`, `sodium`, `sqlite3`, `tokenizer`, `xml`, `xmlreader`, `xmlwriter`, `zip`, `zlib`

These extensions are statically linked and always available — no additional installation needed. However, `pecl`, `phpize`, and `php-config` are **not included** in prebuilt binaries, so installing additional extensions via PECL is not possible.

For the full and up-to-date extension list, see the [static-php-cli extension list](https://static-php.dev/en/guide/extensions.html).

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
- Partial versions like `8.4` resolve to the latest patch (e.g. `8.4.18`)

## Related

- [proto-composer-plugin](https://github.com/KonstantinKai/proto-composer-plugin) — Composer (PHP dependency manager) version management for proto

## License

MIT
