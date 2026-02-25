use crate::config::PhpPluginConfig;
use extism_pdk::*;
use proto_pdk::*;
use std::collections::HashMap;

#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandOutput>;
}

static NAME: &str = "PHP";

#[plugin_fn]
pub fn register_tool(Json(_): Json<RegisterToolInput>) -> FnResult<Json<RegisterToolOutput>> {
    Ok(Json(RegisterToolOutput {
        name: NAME.into(),
        type_of: PluginType::Language,
        minimum_proto_version: Some(Version::new(0, 46, 0)),
        plugin_version: Version::parse(env!("CARGO_PKG_VERSION")).ok(),
        ..RegisterToolOutput::default()
    }))
}

#[plugin_fn]
pub fn define_tool_config(_: ()) -> FnResult<Json<DefineToolConfigOutput>> {
    Ok(Json(DefineToolConfigOutput {
        schema: schematic::SchemaBuilder::build_root::<PhpPluginConfig>(),
    }))
}

#[plugin_fn]
pub fn detect_version_files(_: ()) -> FnResult<Json<DetectVersionOutput>> {
    Ok(Json(DetectVersionOutput {
        files: vec![".php-version".into(), "composer.json".into()],
        ignore: vec!["vendor".into()],
    }))
}

#[plugin_fn]
pub fn parse_version_file(
    Json(input): Json<ParseVersionFileInput>,
) -> FnResult<Json<ParseVersionFileOutput>> {
    let mut version = None;

    if input.file == ".php-version" {
        let trimmed = input.content.trim();
        if !trimmed.is_empty() {
            version = Some(UnresolvedVersionSpec::parse(trimmed)?);
        }
    } else if input.file == "composer.json"
        && let Ok(composer) = serde_json::from_str::<serde_json::Value>(&input.content)
        && let Some(php_req) = composer
            .get("require")
            .and_then(|r| r.get("php"))
            .and_then(|v| v.as_str())
    {
        // Parse "require.php" from composer.json
        // e.g. ">=8.2" -> ">=8.2", "^8.2" -> "^8.2", "8.2.*" -> "~8.2"
        version = UnresolvedVersionSpec::parse(php_req).ok();
    }

    Ok(Json(ParseVersionFileOutput { version }))
}

#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    let tags = load_git_tags("https://github.com/php/php-src")?;

    // PHP tags: php-8.4.3, php-8.4.3RC1, php-8.4.3alpha1, etc.
    let versions: Vec<String> = tags
        .into_iter()
        .filter_map(|tag| {
            let stripped = tag.strip_prefix("php-")?;
            // Only PHP 7.0+
            let first = stripped.chars().next()?;
            if first < '7' {
                return None;
            }
            // Skip pre-release versions (RC, alpha, beta — case-insensitive)
            let lower = stripped.to_lowercase();
            if lower.contains("rc")
                || lower.contains("alpha")
                || lower.contains("beta")
            {
                return None;
            }
            Some(stripped.to_string())
        })
        .collect();

    Ok(Json(LoadVersionsOutput::from(versions)?))
}

#[plugin_fn]
pub fn resolve_version(
    Json(input): Json<ResolveVersionInput>,
) -> FnResult<Json<ResolveVersionOutput>> {
    let mut output = ResolveVersionOutput::default();

    if let UnresolvedVersionSpec::Alias(alias) = input.initial {
        match alias.as_str() {
            "lts" | "stable" => {
                output.candidate = Some(UnresolvedVersionSpec::parse("latest")?);
            }
            _ => {}
        }
    }

    Ok(Json(output))
}

fn map_os(os: &HostOS) -> &'static str {
    match os {
        HostOS::Linux => "linux",
        HostOS::MacOS => "macos",
        HostOS::Windows => "windows",
        _ => "unknown",
    }
}

fn map_arch(arch: &HostArch) -> &'static str {
    match arch {
        HostArch::X64 => "x86_64",
        HostArch::Arm64 => "aarch64",
        _ => "unknown",
    }
}

#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    let env = get_host_environment()?;
    let version = &input.context.version;

    if version.is_canary() {
        return Err(plugin_err!(PluginError::UnsupportedCanary {
            tool: NAME.into(),
        }));
    }

    let config = get_tool_config::<PhpPluginConfig>()?;

    match config.sapi.as_str() {
        "cli" | "fpm" | "micro" => {}
        _ => {
            return Err(plugin_err!(PluginError::Message(format!(
                "Invalid sapi '{}'. Must be one of: cli, fpm, micro",
                config.sapi
            ))));
        }
    }

    if !config.prebuilt {
        return Err(plugin_err!(PluginError::Message(
            "Prebuilt binaries disabled via config. Will build from source.".into(),
        )));
    }

    check_supported_os_and_arch(
        NAME,
        &env,
        permutations![
            HostOS::Linux => [HostArch::X64, HostArch::Arm64],
            HostOS::MacOS => [HostArch::X64, HostArch::Arm64],
            HostOS::Windows => [HostArch::X64],
        ],
    )?;

    let os = map_os(&env.os);
    let arch = map_arch(&env.arch);
    let ver = version.to_string();

    // Windows uses official php.net binaries
    if env.os == HostOS::Windows {
        // Compiler version depends on PHP major.minor:
        // 7.x = vc15, 8.0-8.3 = vs16, 8.4+ = vs17
        let vs = if ver.starts_with("7.") {
            "vc15"
        } else if ver.starts_with("8.0.")
            || ver.starts_with("8.1.")
            || ver.starts_with("8.2.")
            || ver.starts_with("8.3.")
        {
            "vs16"
        } else {
            "vs17"
        };
        let filename = format!("php-{ver}-nts-Win32-{vs}-x64.zip");
        let download_url =
            format!("https://windows.php.net/downloads/releases/{filename}");

        return Ok(Json(DownloadPrebuiltOutput {
            download_url,
            download_name: Some(filename),
            archive_prefix: None,
            ..DownloadPrebuiltOutput::default()
        }));
    }

    // Linux/macOS: static-php-cli prebuilt binaries
    let filename = format!("php-{ver}-{sapi}-{os}-{arch}.tar.gz", sapi = config.sapi);
    let download_url = config
        .dist_url
        .replace("{version}", &ver)
        .replace("{sapi}", &config.sapi)
        .replace("{os}", os)
        .replace("{arch}", arch)
        .replace("{file}", &filename);

    Ok(Json(DownloadPrebuiltOutput {
        download_url,
        download_name: Some(filename),
        archive_prefix: None,
        ..DownloadPrebuiltOutput::default()
    }))
}

#[plugin_fn]
pub fn build_instructions(
    Json(input): Json<BuildInstructionsInput>,
) -> FnResult<Json<BuildInstructionsOutput>> {
    let env = get_host_environment()?;
    let version = &input.context.version;

    if env.os == HostOS::Windows {
        return Err(plugin_err!(PluginError::Message(
            "Building PHP from source is not supported on Windows. Use prebuilt binaries."
                .into(),
        )));
    }

    let config = get_tool_config::<PhpPluginConfig>()?;

    let prefix = format!("--prefix={}", input.install_dir);
    let mut configure_args: Vec<&str> = vec![
        &prefix,
        "--enable-mbstring",
        "--enable-bcmath",
        "--enable-pcntl",
        "--with-openssl",
        "--with-curl",
        "--with-zlib",
        "--with-pdo-sqlite",
    ];

    let extra_opts;
    if let Some(ref opts) = config.configure_opts {
        extra_opts = opts.clone();
        configure_args.extend(extra_opts.iter().map(|s| s.as_str()));
    }

    Ok(Json(BuildInstructionsOutput {
        help_url: Some("https://www.php.net/manual/en/install.unix.php".into()),
        source: Some(SourceLocation::Archive(ArchiveSource {
            url: format!("https://www.php.net/distributions/php-{version}.tar.xz"),
            prefix: Some(format!("php-{version}")),
        })),
        system_dependencies: vec![
            SystemDependency::for_pm(
                HostPackageManager::Apt,
                [
                    "build-essential",
                    "autoconf",
                    "bison",
                    "re2c",
                    "pkg-config",
                    "libxml2-dev",
                    "libsqlite3-dev",
                    "libssl-dev",
                    "libcurl4-openssl-dev",
                    "libonig-dev",
                    "libreadline-dev",
                    "libzip-dev",
                    "zlib1g-dev",
                    "libicu-dev",
                ],
            ),
            SystemDependency::for_pm(
                HostPackageManager::Brew,
                [
                    "autoconf",
                    "bison",
                    "re2c",
                    "pkg-config",
                    "libxml2",
                    "openssl@3",
                    "curl",
                    "oniguruma",
                    "readline",
                    "libzip",
                    "zlib",
                    "icu4c",
                ],
            ),
            SystemDependency::for_pm(
                HostPackageManager::Dnf,
                [
                    "gcc",
                    "gcc-c++",
                    "make",
                    "autoconf",
                    "bison",
                    "re2c",
                    "pkgconf",
                    "libxml2-devel",
                    "sqlite-devel",
                    "openssl-devel",
                    "libcurl-devel",
                    "oniguruma-devel",
                    "readline-devel",
                    "libzip-devel",
                    "zlib-devel",
                    "libicu-devel",
                ],
            ),
        ],
        requirements: vec![
            BuildRequirement::XcodeCommandLineTools,
            BuildRequirement::CommandExistsOnPath("make".into()),
            BuildRequirement::CommandExistsOnPath("autoconf".into()),
        ],
        instructions: vec![
            BuildInstruction::RunCommand(Box::new(CommandInstruction::new(
                "./configure",
                configure_args,
            ))),
            BuildInstruction::RunCommand(Box::new(CommandInstruction::new(
                "make",
                ["-j4"],
            ))),
            BuildInstruction::RunCommand(Box::new(CommandInstruction::new(
                "make",
                ["install"],
            ))),
        ],
    }))
}

#[plugin_fn]
pub fn locate_executables(
    Json(_): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_host_environment()?;

    // NOTE: static-php-cli prebuilt archives contain only `php` at the root.
    // Windows php.net archives contain `php.exe` at the root.
    // Build-from-source puts executables in `bin/`.
    let primary = if env.os == HostOS::Windows {
        env.os.get_exe_name("php")
    } else {
        "php".into()
    };

    let exes = HashMap::from_iter([(
        "php".into(),
        ExecutableConfig::new_primary(primary),
    )]);

    Ok(Json(LocateExecutablesOutput {
        exes,
        exes_dirs: vec![".".into(), "bin".into()],
        globals_lookup_dirs: vec![
            "$HOME/.composer/vendor/bin".into(),
            "$COMPOSER_HOME/vendor/bin".into(),
        ],
        ..LocateExecutablesOutput::default()
    }))
}

#[plugin_fn]
pub fn sync_shell_profile(
    Json(_): Json<SyncShellProfileInput>,
) -> FnResult<Json<SyncShellProfileOutput>> {
    Ok(Json(SyncShellProfileOutput {
        check_var: "PROTO_PHP_VERSION".into(),
        export_vars: Some(HashMap::from_iter([(
            "PHP_HOME".into(),
            "$PROTO_HOME/tools/php/$PROTO_PHP_VERSION".into(),
        )])),
        extend_path: Some(vec!["$HOME/.composer/vendor/bin".into()]),
        skip_sync: false,
    }))
}
