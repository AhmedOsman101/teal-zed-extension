# Teal for Zed

Teal language support for Zed: syntax highlighting, brackets, indentation, outline, Vim text objects, and `teal-language-server` integration. The grammar is pinned to [tree-sitter-teal](https://github.com/euclidianAce/tree-sitter-teal) at `ac6b42ff` (ABI 15).

## Requirements

- Zed 1.23.2 or newer.
- `teal-language-server` for diagnostics, hover, definition, and completion:

  ```sh
  luarocks install teal-language-server
  ```

- Optional: `tl` to run and check Teal code, and `ceru` for formatting.

## Install

This version is installed as a dev extension:

1. Open the Extensions page.
2. Choose **Install Dev Extension**.
3. Select this repository.

Registry publication is not part of this version. Before submitting to the registry, add a top-level `repository` URL to `extension.toml`.

## Language server

The extension resolves the server in this order:

1. `lsp.teal-language-server.binary.path` from Zed settings.
2. `teal-language-server` on `PATH`.
3. The LuaRocks user tree:
   - Linux and macOS: `~/.luarocks/bin/teal-language-server`.
   - Windows: `%APPDATA%\luarocks\bin\teal-language-server.bat`, then `~/.luarocks/bin/teal-language-server.bat`.

The Windows fallback covers native LuaRocks and MSYS2-MinGW layouts. Pure MSYS/Cygwin LuaRocks installs a suffixless wrapper under `~/.luarocks/bin`; set `lsp.teal-language-server.binary.path` or put that directory on `PATH` for those installations.

If none exists, Zed shows an error naming the binary, the install command, and the upstream repository.

The default arguments are `-d INFO`. Do not add `-L` or `--log-mode`: version 0.2.1 crashes on shutdown with `attempt to use a closed file`.

Settings example:

```json
{
  "lsp": {
    "teal-language-server": {
      "binary": {
        "path": "/usr/local/bin/teal-language-server",
        "arguments": ["-d", "INFO"],
        "env": {
          "LUA_PATH": "/custom/?.lua;;"
        }
      }
    }
  }
}
```

`binary.env` is passed through to the server unchanged. `tlconfig.lua` is optional. Without it, the server still diagnoses open files; with it, `source_dir`, `build_dir`, and `include_dir` are honored.

## Formatter

Zed does not ship a Teal formatter in this extension. Use `ceru` as an external formatter:

```json
{
  "languages": {
    "Teal": {
      "formatter": {
        "external": {
          "command": "ceru",
          "arguments": ["--indent", "2", "--line-length", "100", "-"]
        }
      }
    }
  }
}
```

`ceru` reads the buffer on stdin and writes the formatted buffer to stdout, which is what Zed expects. Use `--indent 3` to match upstream Teal's `.editorconfig`; the value above matches the author's Helix setup.

## What's included

- `languages/teal/highlights.scm`
- `languages/teal/brackets.scm`
- `languages/teal/indents.scm`
- `languages/teal/outline.scm`
- `languages/teal/textobjects.scm`
- `examples/sample.tl` as a manual fixture.

The language config deliberately does not set `tab_size`; Zed uses your global editor setting. Folding is derived from indentation because Zed has no `folds.scm`. For non-empty bodies, text objects select the inside of function, record, interface, and enum bodies without the closing `end`; empty bodies fall back to the around range.

## Troubleshooting

- **`teal-language-server was not found`**: install it with `luarocks install teal-language-server`, put it on `PATH`, or set `lsp.teal-language-server.binary.path`.
- **No diagnostics**: open a `.tl` file inside a project and check `zed --foreground` for server logs. `tlconfig.lua` is optional.
- **Grammar ABI error**: the pinned grammar is ABI 15. No verified ABI-14 tag exists in the upstream repository (`v0.1.0` is ABI 15 and `v0.0.4` is ABI 12). If your Zed rejects ABI 15, report the Zed version rather than silently downgrading.
- **`wasm-component-ld` rejects `-fuse-ld=mold`**: if your shell exports `RUSTFLAGS` containing `-fuse-ld=mold`, build with `RUSTFLAGS= cargo build --release --target wasm32-wasip2`, or unset `RUSTFLAGS` for Zed before installing the dev extension.
- **Completion after a bare `.` is empty**: server 0.2.1 returns `(none)` at a dot; keep typing (`p.x`) and field completion appears.

## License

Apache-2.0. See [LICENSE](LICENSE).
