/// Default extensions matching the static-php-cli prebuilt binaries.
/// See: https://dl.static-php.dev/static-php-cli/common/README.txt
pub const DEFAULT_EXTENSIONS: &[&str] = &[
    "bcmath",
    "bz2",
    "calendar",
    "ctype",
    "curl",
    "dom",
    "exif",
    "fileinfo",
    "filter",
    "ftp",
    "gd",
    "gmp",
    "iconv",
    "mbregex",
    "mbstring",
    "mysqlnd",
    "openssl",
    "pcntl",
    "pdo",
    "pdo_mysql",
    "pdo_pgsql",
    "pdo_sqlite",
    "pgsql",
    "phar",
    "posix",
    "redis",
    "session",
    "simplexml",
    "soap",
    "sockets",
    "sqlite3",
    "tokenizer",
    "xml",
    "xmlreader",
    "xmlwriter",
    "zip",
    "zlib",
];

/// Configuration for the PHP proto plugin.
/// Users can override these in `.prototools` under `[tools.php]`.
#[derive(Debug, schematic::Schematic, serde::Deserialize, serde::Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct PhpPluginConfig {
    /// Download prebuilt static binaries instead of building from source (default: true).
    /// When false, PHP 8.x builds use spc (static-php-cli) and PHP 7.x uses php.net tarballs.
    pub prebuilt: bool,

    /// SAPI type: "cli", "fpm", or "micro" (default: "cli").
    /// Applies to prebuilt binaries and spc source builds (Linux/macOS only).
    pub sapi: String,

    /// Download URL template for prebuilt binaries.
    /// Supports placeholders: {version}, {sapi}, {os}, {arch}, {file}
    pub dist_url: String,

    /// Extra flags passed to `./configure` when building from source.
    pub configure_opts: Option<Vec<String>>,

    /// Extensions for spc (static-php-cli) source builds.
    /// Defaults to the 37 extensions matching prebuilt binaries.
    pub extensions: Option<Vec<String>>,
}

impl Default for PhpPluginConfig {
    fn default() -> Self {
        Self {
            prebuilt: true,
            sapi: "cli".into(),
            dist_url: "https://dl.static-php.dev/static-php-cli/common/php-{version}-{sapi}-{os}-{arch}.tar.gz".into(),
            configure_opts: None,
            extensions: None,
        }
    }
}

impl PhpPluginConfig {
    /// Returns the configured extensions or the default set.
    pub fn effective_extensions(&self) -> Vec<String> {
        self.extensions
            .clone()
            .unwrap_or_else(|| DEFAULT_EXTENSIONS.iter().map(|s| s.to_string()).collect())
    }
}
