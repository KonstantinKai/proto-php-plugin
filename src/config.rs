/// Configuration for the PHP proto plugin.
/// Users can override these in `.prototools` under `[tools.php]`.
#[derive(Debug, schematic::Schematic, serde::Deserialize, serde::Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct PhpPluginConfig {
    /// Use static-php-cli prebuilt binaries (default: true).
    /// Set to false to build from source via php.net tarballs.
    pub prebuilt: bool,

    /// SAPI type for prebuilt binaries: "cli", "fpm", or "micro" (default: "cli").
    pub sapi: String,

    /// Download URL template for prebuilt binaries.
    /// Supports placeholders: {version}, {sapi}, {os}, {arch}, {file}
    pub dist_url: String,

    /// Extra flags passed to `./configure` when building from source.
    pub configure_opts: Option<Vec<String>>,
}

impl Default for PhpPluginConfig {
    fn default() -> Self {
        Self {
            prebuilt: true,
            sapi: "cli".into(),
            dist_url: "https://dl.static-php.dev/static-php-cli/common/php-{version}-{sapi}-{os}-{arch}.tar.gz".into(),
            configure_opts: None,
        }
    }
}
