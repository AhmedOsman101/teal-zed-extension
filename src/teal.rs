use zed_extension_api::{
    self as zed, settings::LspSettings, LanguageServerId, Os, Result, Worktree,
};

fn default_args(configured: Option<Vec<String>>) -> Vec<String> {
    configured.unwrap_or_else(|| vec!["-d".into(), "INFO".into()])
}

fn home_server_path(shell_env: &[(String, String)], os: Os) -> Option<String> {
    let home = shell_env
        .iter()
        .find(|(name, _)| name == "HOME")
        .or_else(|| shell_env.iter().find(|(name, _)| name == "USERPROFILE"))
        .map(|(_, value)| value.as_str())?;
    let suffix = if matches!(os, Os::Windows) {
        ".exe"
    } else {
        ""
    };
    Some(format!("{home}/.luarocks/bin/teal-language-server{suffix}"))
}

fn resolve_server(
    configured_path: Option<String>,
    configured_args: Option<Vec<String>>,
    shell_env: &[(String, String)],
    os: Os,
    which: impl Fn(&str) -> Option<String>,
) -> Result<zed::Command> {
    let command = configured_path
        .or_else(|| which("teal-language-server"))
        .or_else(|| home_server_path(shell_env, os).and_then(|path| which(&path)))
        .ok_or_else(missing_server_error)?;

    Ok(zed::Command {
        command,
        args: default_args(configured_args),
        env: vec![],
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
        let (configured_path, configured_args) = match binary {
            Some(binary) => (binary.path, binary.arguments),
            None => (None, None),
        };

        let shell_env = worktree.shell_env();
        let (os, _) = zed::current_platform();
        resolve_server(configured_path, configured_args, &shell_env, os, |name| {
            worktree.which(name)
        })
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
    fn home_candidate_uses_home() {
        let env = vec![("HOME".into(), "/home/u".into())];
        assert_eq!(
            home_server_path(&env, Os::Linux).as_deref(),
            Some("/home/u/.luarocks/bin/teal-language-server")
        );
    }

    #[test]
    fn home_candidate_falls_back_to_userprofile() {
        let env = vec![("USERPROFILE".into(), "C:\\Users\\u".into())];
        assert_eq!(
            home_server_path(&env, Os::Windows).as_deref(),
            Some("C:\\Users\\u/.luarocks/bin/teal-language-server.exe")
        );
    }

    #[test]
    fn home_candidate_prefers_home_over_userprofile() {
        let env = vec![
            ("USERPROFILE".into(), "C:\\Users\\u".into()),
            ("HOME".into(), "/home/u".into()),
        ];
        assert_eq!(
            home_server_path(&env, Os::Linux).as_deref(),
            Some("/home/u/.luarocks/bin/teal-language-server")
        );
    }

    #[test]
    fn home_candidate_is_none_without_home_variables() {
        assert_eq!(home_server_path(&[], Os::Linux), None);
    }

    #[test]
    fn configured_path_wins_without_calling_which() {
        let command = resolve_server(
            Some("/custom/teal-language-server".into()),
            None,
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
        let command = resolve_server(None, None, &[], Os::Linux, |name| {
            (name == "teal-language-server").then(|| "/usr/bin/teal-language-server".into())
        })
        .unwrap();
        assert_eq!(command.command, "/usr/bin/teal-language-server");
    }

    #[test]
    fn luarocks_fallback_is_resolved_through_which() {
        let env = vec![("HOME".into(), "/home/u".into())];
        let command = resolve_server(None, None, &env, Os::Linux, |name| {
            (name == "/home/u/.luarocks/bin/teal-language-server").then(|| name.into())
        })
        .unwrap();
        assert_eq!(
            command.command,
            "/home/u/.luarocks/bin/teal-language-server"
        );
    }

    #[test]
    fn missing_server_error_is_actionable() {
        let error = resolve_server(None, None, &[], Os::Linux, |_| None).unwrap_err();
        assert!(error.contains("teal-language-server was not found"));
        assert!(error.contains("luarocks install teal-language-server"));
        assert!(error.contains("lsp.teal-language-server.binary.path"));
        assert!(error.contains("https://github.com/teal-language/teal-language-server"));
        assert_eq!(error, missing_server_error());
    }
}
