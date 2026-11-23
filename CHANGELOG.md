# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Windows prebuilt binaries for PHP 8.6, which windows.php.net builds with Visual Studio 2026 (`vs18`) ([#2](https://github.com/KonstantinKai/proto-php-plugin/pull/2)).

## [0.5.0] - 2026-08-26

### Breaking

- Requires proto >= 0.56.0. Older versions do not send the `working_dir` context field the plugin now expects.

### Fixed

- Fixed the `missing field tool_dir` error on recent proto releases, which removed the deprecated `tool_dir` field from the unresolved plugin context.

### Changed

- Updated to `proto_pdk` 0.35, `proto_pdk_test_utils` 0.49, and `schematic` 0.20.
- Replaced the removed `VirtualPath::real_path` with `to_real_path`, which now returns a `Result` of an optional `RealPath`.
- Filled in the new `BuildInstructionsOutput.http_headers` field via the struct default.

## [0.4.0] - 2026-02-25

### Added

- PHP 8.x source builds now use [static-php-cli](https://static-php.dev) (`spc`) to produce fully static binaries with configurable extensions and SAPI support. PHP 7.x and custom `configure-opts` fall back to the traditional php.net `configure`/`make` build.
- `extensions` config option to customize the extension list for spc source builds (defaults to the 37 extensions matching the prebuilt binaries).
- `spc doctor --auto-fix` runs during spc builds to install missing build dependencies.
- README troubleshooting sections for common build issues: PHP 7.x on macOS (OpenSSL 3.x, DNS resolver linking, PCRE JIT allocation), iconv on macOS, and the GitHub API rate limit (403) in spc.

### Changed

- `sapi` applies to prebuilt binaries and spc source builds; the php.net build path uses `configure-opts` instead.

### Fixed

- php.net builds passed the virtual WASI path to `--prefix` instead of the real filesystem path.

## [0.3.0] - 2026-02-25

### Added

- `sapi` config option to install `fpm` or `micro` binaries from static-php-cli instead of only `cli`. Supported values: `cli` (default), `fpm`, `micro`. Only applies to Linux/macOS prebuilt binaries.
- `dist-url` template supports a `{sapi}` placeholder.
- Invalid `sapi` values produce a clear error message.

### Removed

- SHA-256 checksum verification (`releases.json`, probe script, weekly update workflow). Maintaining a separate index per version, platform and SAPI added complexity without enough benefit.

## [0.2.0] - 2026-02-24

### Added

- `download_prebuilt` checks a releases index to confirm a prebuilt binary exists for the requested version and platform before downloading.
- SHA-256 checksum verification of downloads using checksums from the releases index.
- Error messages for unavailable versions or platforms suggest nearby versions and the `--build` flag.
- CI, Release and License badges in the README.

### Fixed

- Prebuilt extension list now matches the [static-php-cli source](https://dl.static-php.dev/static-php-cli/common/README.txt) (37 extensions).

## [0.1.0] - 2026-02-24

### Added

- Install PHP versions via `proto install php 8.4`.
- Prebuilt static binaries from [static-php-cli](https://static-php.dev) for Linux and macOS (x64 and arm64).
- Windows support via official [windows.php.net](https://windows.php.net) binaries.
- Build from source using official php.net tarballs (`proto install php 8.4 --build`).
- Version detection from `.php-version` and `composer.json` (`require.php`).
- Configurable `./configure` flags for source builds.

[Unreleased]: https://github.com/KonstantinKai/proto-php-plugin/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/KonstantinKai/proto-php-plugin/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/KonstantinKai/proto-php-plugin/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/KonstantinKai/proto-php-plugin/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/KonstantinKai/proto-php-plugin/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/KonstantinKai/proto-php-plugin/releases/tag/v0.1.0
