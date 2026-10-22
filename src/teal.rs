use zed_extension_api::{
    self as zed, settings::LspSettings, LanguageServerId, Os, Result, Worktree,
};

fn default_args(configured: Option<Vec<String>>) -> Vec<String> {
    configured.unwrap_or_else(|| vec!["-d".into(), "INFO".into()])
}

fn env_value<'a>(shell_env: &'a [(String, String)], name: &str) -> Option<&'a str> {
    shell_env
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

fn luarocks_fallback_paths(shell_env: &[(String, String)], os: Os) -> Vec<String> {
    if matches!(os, Os::Windows) {
        let mut paths = Vec::new();

        if let Some(appdata) = env_value(shell_env, "APPDATA") {
            paths.push(format!("{appdata}/luarocks/bin/teal-language-server.bat"));
        }

        if let Some(home) =
            env_value(shell_env, "HOME").or_else(|| env_value(shell_env, "USERPROFILE"))
        {
            paths.push(format!("{home}/.luarocks/bin/teal-language-server.bat"));
        }

        paths
    } else {
        env_value(shell_env, "HOME")
            .or_else(|| env_value(shell_env, "USERPROFILE"))
            .map(|home| vec![format!("{home}/.luarocks/bin/teal-language-server")])
            .unwrap_or_default()
    }
}

fn resolve_server(
    configured_path: Option<String>,
    configured_args: Option<Vec<String>>,
    configured_env: Vec<(String, String)>,
    shell_env: &[(String, String)],
    os: Os,
    which: impl Fn(&str) -> Option<String>,
) -> Result<zed::Command> {
    let command = configured_path
        .or_else(|| which("teal-language-server"))
        .or_else(|| {
            luarocks_fallback_paths(shell_env, os)
                .into_iter()
                .find_map(|path| which(&path))
        })
        .ok_or_else(missing_server_error)?;

    Ok(zed::Command {
        command,
        args: default_args(configured_args),
        env: configured_env,
    })
}

fn missing_server_error() -> String {
    r"teal-language-server was not found. Install it with `luarocks install teal-language-server`, or set `lsp.teal-language-server.binary.path` in Zed settings. See https://github.com/teal-language/teal-language-server.".to_string()
}

struct TealExtension;

impl zed::Extension for TealExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<zed::Command> {
        let binary = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|settings| settings.binary);
        let (configured_path, configured_args, configured_env) = match binary {
            Some(binary) => (binary.path, binary.arguments, binary.env),
            None => (None, None, None),
        };
        let configured_env = configured_env
            .map(|env| {
                let mut env: Vec<_> = env.into_iter().collect();
                env.sort();
                env
            })
            .unwrap_or_default();

        let shell_env = worktree.shell_env();
        let (os, _) = zed::current_platform();
        resolve_server(
            configured_path,
            configured_args,
            configured_env,
            &shell_env,
            os,
            |name| worktree.which(name),
        )
    }
}

zed::register_extension!(TealExtension);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_arguments_are_used_verbatim() {
        assert_eq!(
            default_args(Some(vec!["--custom".into(), "value".into()])),
            vec!["--custom", "value"]
        );
        assert!(default_args(Some(vec![])).is_empty());
    }

    #[test]
    fn default_arguments_enable_info_logging() {
        assert_eq!(default_args(None), vec!["-d", "INFO"]);
    }

    #[test]
    fn luarocks_fallback_uses_home() {
        let env = vec![("HOME".into(), "/home/u".into())];
        assert_eq!(
            luarocks_fallback_paths(&env, Os::Linux),
            vec!["/home/u/.luarocks/bin/teal-language-server"]
        );
    }

    #[test]
    fn luarocks_fallback_uses_userprofile_on_unix() {
        let env = vec![("USERPROFILE".into(), "C:\\Users\\u".into())];
        assert_eq!(
            luarocks_fallback_paths(&env, Os::Linux),
            vec!["C:\\Users\\u/.luarocks/bin/teal-language-server"]
        );
    }

    #[test]
    fn luarocks_fallback_prefers_home_over_userprofile() {
        let env = vec![
            ("USERPROFILE".into(), "C:\\Users\\u".into()),
            ("HOME".into(), "/home/u".into()),
        ];
        assert_eq!(
            luarocks_fallback_paths(&env, Os::Linux),
            vec!["/home/u/.luarocks/bin/teal-language-server"]
        );
    }

    #[test]
    fn windows_luarocks_fallback_uses_appdata_bat() {
        let env = vec![("APPDATA".into(), "C:\\Users\\u\\AppData\\Roaming".into())];
        assert_eq!(
            luarocks_fallback_paths(&env, Os::Windows),
            vec!["C:\\Users\\u\\AppData\\Roaming/luarocks/bin/teal-language-server.bat"]
        );
    }

    #[test]
    fn windows_luarocks_fallback_includes_home_bat_candidate() {
        let env = vec![
            ("APPDATA".into(), "C:\\Users\\u\\AppData\\Roaming".into()),
            ("HOME".into(), "C:\\msys64\\home\\u".into()),
        ];
        assert_eq!(
            luarocks_fallback_paths(&env, Os::Windows),
            vec![
                "C:\\Users\\u\\AppData\\Roaming/luarocks/bin/teal-language-server.bat",
                "C:\\msys64\\home\\u/.luarocks/bin/teal-language-server.bat",
            ]
        );
    }

    #[test]
    fn luarocks_fallback_is_empty_without_home_variables() {
        assert!(luarocks_fallback_paths(&[], Os::Linux).is_empty());
        assert!(luarocks_fallback_paths(&[], Os::Windows).is_empty());
    }

    #[test]
    fn configured_path_wins_without_calling_which() {
        let command = resolve_server(
            Some("/custom/teal-language-server".into()),
            None,
            vec![],
            &[],
            Os::Linux,
            |_| panic!("configured path must not be resolved through which"),
        )
        .unwrap();
        assert_eq!(command.command, "/custom/teal-language-server");
        assert_eq!(command.args, vec!["-d", "INFO"]);
        assert!(command.env.is_empty());
    }

    #[test]
    fn path_lookup_wins_before_the_luarocks_fallback() {
        let command = resolve_server(None, None, vec![], &[], Os::Linux, |name| {
            if name == "teal-language-server" {
                Some("/usr/bin/teal-language-server".into())
            } else {
                panic!("luarocks fallback must not be resolved when PATH wins")
            }
        })
        .unwrap();
        assert_eq!(command.command, "/usr/bin/teal-language-server");
    }

    #[test]
    fn luarocks_fallback_is_resolved_through_which() {
        let env = vec![("HOME".into(), "/home/u".into())];
        let command = resolve_server(None, None, vec![], &env, Os::Linux, |name| {
            (name == "/home/u/.luarocks/bin/teal-language-server").then(|| name.into())
        })
        .unwrap();
        assert_eq!(
            command.command,
            "/home/u/.luarocks/bin/teal-language-server"
        );
    }

    #[test]
    fn windows_luarocks_fallback_is_resolved_through_which() {
        let env = vec![("APPDATA".into(), "C:\\Users\\u\\AppData\\Roaming".into())];
        let command = resolve_server(None, None, vec![], &env, Os::Windows, |name| {
            (name == "C:\\Users\\u\\AppData\\Roaming/luarocks/bin/teal-language-server.bat")
                .then(|| name.into())
        })
        .unwrap();
        assert_eq!(
            command.command,
            "C:\\Users\\u\\AppData\\Roaming/luarocks/bin/teal-language-server.bat"
        );
    }

    #[test]
    fn configured_environment_is_propagated() {
        let command = resolve_server(
            None,
            None,
            vec![("LUA_PATH".into(), "/custom/?.lua".into())],
            &[],
            Os::Linux,
            |name| (name == "teal-language-server").then(|| "/usr/bin/teal-language-server".into()),
        )
        .unwrap();
        assert_eq!(
            command.env,
            vec![("LUA_PATH".to_string(), "/custom/?.lua".to_string())]
        );
    }

    #[test]
    fn missing_server_error_is_actionable() {
        let error = resolve_server(None, None, vec![], &[], Os::Linux, |_| None).unwrap_err();
        assert!(error.contains("teal-language-server was not found"));
        assert!(error.contains("luarocks install teal-language-server"));
        assert!(error.contains("lsp.teal-language-server.binary.path"));
        assert!(error.contains("https://github.com/teal-language/teal-language-server"));
        assert_eq!(error, missing_server_error());
    }
}
