use proto_pdk_test_utils::*;

mod php_tool {
    use super::*;

    generate_resolve_versions_tests!("php-test", {
        "8.4" => "8.4.18",
    });

    #[tokio::test(flavor = "multi_thread")]
    async fn registers_tool_metadata() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("php-test").await;
        let output = plugin
            .register_tool(RegisterToolInput {
                id: Id::raw("php-test"),
                ..Default::default()
            })
            .await;

        assert_eq!(output.name, "PHP");
        assert_eq!(output.type_of, PluginType::Language);
        assert!(output.minimum_proto_version.is_some());
        assert!(output.plugin_version.is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn locates_executables_unix() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox
            .create_plugin_with_config("php-test", |config| {
                config.host(HostOS::Linux, HostArch::X64);
            })
            .await;

        let output = plugin
            .locate_executables(LocateExecutablesInput {
                context: PluginContext {
                    version: VersionSpec::parse("8.4.3").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;

        let php = output.exes.get("php").expect("php executable missing");
        assert!(php.primary);
        assert_eq!(
            php.exe_path.as_deref(),
            Some(std::path::Path::new("php"))
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn loads_versions_from_git() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("php-test").await;
        let output = plugin.load_versions(LoadVersionsInput::default()).await;

        assert!(!output.versions.is_empty());

        // Should contain recent PHP versions
        let version_strings: Vec<String> =
            output.versions.iter().map(|v| v.to_string()).collect();
        assert!(version_strings.iter().any(|v| v.starts_with("8.4")));
        assert!(version_strings.iter().any(|v| v.starts_with("8.3")));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn detects_version_files() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("php-test").await;
        let output = plugin.detect_version_files(DetectVersionInput::default()).await;

        assert_eq!(output.files, vec![".php-version", "composer.json"]);
        assert_eq!(output.ignore, vec!["vendor"]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn parses_php_version_file() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("php-test").await;

        let output = plugin
            .parse_version_file(ParseVersionFileInput {
                content: "8.4.3\n".into(),
                file: ".php-version".into(),
                ..Default::default()
            })
            .await;

        assert_eq!(
            output.version,
            Some(UnresolvedVersionSpec::parse("8.4.3").unwrap())
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn parses_composer_json_gte() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("php-test").await;

        let output = plugin
            .parse_version_file(ParseVersionFileInput {
                content: r#"{"require":{"php":">=8.2"}}"#.into(),
                file: "composer.json".into(),
                ..Default::default()
            })
            .await;

        assert!(output.version.is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn parses_composer_json_caret() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("php-test").await;

        let output = plugin
            .parse_version_file(ParseVersionFileInput {
                content: r#"{"require":{"php":"^8.2"}}"#.into(),
                file: "composer.json".into(),
                ..Default::default()
            })
            .await;

        assert!(output.version.is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn parses_composer_json_or_constraint() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("php-test").await;

        let output = plugin
            .parse_version_file(ParseVersionFileInput {
                content: r#"{"require":{"php":">=8.1 || >=8.2"}}"#.into(),
                file: "composer.json".into(),
                ..Default::default()
            })
            .await;

        assert!(output.version.is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ignores_composer_json_without_php() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("php-test").await;

        let output = plugin
            .parse_version_file(ParseVersionFileInput {
                content: r#"{"require":{"ext-json":"*"}}"#.into(),
                file: "composer.json".into(),
                ..Default::default()
            })
            .await;

        assert!(output.version.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn download_url_linux_x64() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox
            .create_plugin_with_config("php-test", |config| {
                config.host(HostOS::Linux, HostArch::X64);
            })
            .await;

        let output = plugin
            .download_prebuilt(DownloadPrebuiltInput {
                context: PluginContext {
                    version: VersionSpec::parse("8.4.18").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;

        assert!(output.download_url.contains("linux"));
        assert!(output.download_url.contains("x86_64"));
        assert!(output.download_url.contains("8.4.18"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn download_url_macos_arm64() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox
            .create_plugin_with_config("php-test", |config| {
                config.host(HostOS::MacOS, HostArch::Arm64);
            })
            .await;

        let output = plugin
            .download_prebuilt(DownloadPrebuiltInput {
                context: PluginContext {
                    version: VersionSpec::parse("8.4.18").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;

        assert!(output.download_url.contains("macos"));
        assert!(output.download_url.contains("aarch64"));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore] // Requires remote releases.json in new format (with checksums)
    #[should_panic(expected = "does not have prebuilt binaries available")]
    async fn releases_index_rejects_missing_version() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox
            .create_plugin_with_config("php-test", |config| {
                config.host(HostOS::Linux, HostArch::X64);
            })
            .await;

        // 8.4.2 is not in the releases index
        plugin
            .download_prebuilt(DownloadPrebuiltInput {
                context: PluginContext {
                    version: VersionSpec::parse("8.4.2").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn releases_index_allows_known_version() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox
            .create_plugin_with_config("php-test", |config| {
                config.host(HostOS::Linux, HostArch::X64);
            })
            .await;

        // 8.4.18 is in the releases index with linux-x86_64
        let output = plugin
            .download_prebuilt(DownloadPrebuiltInput {
                context: PluginContext {
                    version: VersionSpec::parse("8.4.18").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;

        assert!(output.download_url.contains("8.4.18"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn releases_index_skipped_for_windows() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox
            .create_plugin_with_config("php-test", |config| {
                config.host(HostOS::Windows, HostArch::X64);
            })
            .await;

        // Windows bypasses releases index (uses windows.php.net)
        let output = plugin
            .download_prebuilt(DownloadPrebuiltInput {
                context: PluginContext {
                    version: VersionSpec::parse("8.4.3").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;

        assert!(output.download_url.contains("windows.php.net"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn download_url_windows_vs17() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox
            .create_plugin_with_config("php-test", |config| {
                config.host(HostOS::Windows, HostArch::X64);
            })
            .await;

        let output = plugin
            .download_prebuilt(DownloadPrebuiltInput {
                context: PluginContext {
                    version: VersionSpec::parse("8.4.3").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;

        assert!(output.download_url.contains("windows.php.net"));
        assert!(output.download_url.contains("vs17-x64.zip"));
    }

    // Integration tests: actual download + install
    generate_download_install_tests!("php-test", "8.4.12");

    generate_shims_test!("php-test", ["php"]);

    #[tokio::test(flavor = "multi_thread")]
    async fn switches_between_versions() {
        let sandbox = create_empty_proto_sandbox();
        let mut plugin = create_plugin!(sandbox, "php-test", None, |_| {});

        // Install 8.4.12
        let mut spec_a = ToolSpec::parse("8.4.12").unwrap();
        flow::manage::Manager::new(&mut plugin.tool)
            .install(&mut spec_a, flow::install::InstallOptions::default())
            .await
            .unwrap();

        let dir_a = plugin.tool.get_product_dir(&spec_a);
        assert!(dir_a.exists(), "8.4.12 product dir should exist");

        // Install 8.3.20
        let mut spec_b = ToolSpec::parse("8.3.20").unwrap();
        flow::manage::Manager::new(&mut plugin.tool)
            .install(&mut spec_b, flow::install::InstallOptions::default())
            .await
            .unwrap();

        let dir_b = plugin.tool.get_product_dir(&spec_b);
        assert!(dir_b.exists(), "8.3.20 product dir should exist");

        // Both versions should coexist
        assert!(dir_a.exists(), "8.4.12 should still exist after installing 8.3.20");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn build_instructions_returns_valid_output() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox
            .create_plugin_with_config("php-test", |config| {
                config.host(HostOS::Linux, HostArch::X64);
            })
            .await;

        let output: BuildInstructionsOutput = plugin
            .tool
            .plugin
            .call_func_with(
                PluginFunction::BuildInstructions,
                BuildInstructionsInput {
                    context: PluginContext {
                        version: VersionSpec::parse("8.4.3").unwrap(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // Source tarball from php.net
        let source = output.source.expect("source should be set");
        match source {
            SourceLocation::Archive(archive) => {
                assert!(archive.url.contains("php.net/distributions"));
                assert!(archive.url.contains("8.4.3"));
                assert!(archive.url.ends_with(".tar.xz"));
                assert_eq!(archive.prefix, Some("php-8.4.3".into()));
            }
            _ => panic!("expected SourceLocation::Archive"),
        }

        // Should have system dependencies
        assert!(!output.system_dependencies.is_empty());

        // Should have build instructions (configure, make, make install)
        assert_eq!(output.instructions.len(), 3);

        // Should have help URL
        assert!(output.help_url.is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn download_url_windows_vs16() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox
            .create_plugin_with_config("php-test", |config| {
                config.host(HostOS::Windows, HostArch::X64);
            })
            .await;

        let output = plugin
            .download_prebuilt(DownloadPrebuiltInput {
                context: PluginContext {
                    version: VersionSpec::parse("8.2.10").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;

        assert!(output.download_url.contains("vs16-x64.zip"));
    }
}
