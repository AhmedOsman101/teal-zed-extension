# Teal Zed Extension Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a Zed extension that adds Teal (`.tl`) support: syntax highlighting, brackets, indentation, outline, text objects, and a `teal-language-server` LSP connection.

**Architecture:** A Zed language extension with a pinned tree-sitter grammar and five query files. A Rust cdylib, compiled to `wasm32-wasip2`, resolves and launches `teal-language-server` from Zed's LSP settings, `$PATH`, or the LuaRocks user tree. A checked-in fixture exercises every query and the LSP.

**Tech Stack:** Zed 1.23.2, `zed_extension_api` 0.7.0, Rust edition 2021 (`cdylib`, `wasm32-wasip2`), `euclidianAce/tree-sitter-teal`, `teal-language-server` 0.2.1, `ceru` 1.9.1 (documented as an external formatter only).

**Spec:** Inline — the "Design decisions" section below is the agreed spec from the grilling session.

## Design Decisions

These are settled. Do not reopen them while implementing.

1. **Scope:** v1 ships language support plus the LSP. Automatic installation of `teal-language-server` is explicitly deferred; a missing server produces an actionable error.
2. **Grammar pin:** `https://github.com/euclidianAce/tree-sitter-teal` at `ac6b42ffb23d5d52b582c02508dd3524ac0ba3df` (ABI 15, 2026-08-13). It includes the function-statement-without-scope and generic-type fixes.
3. **Indentation:** do not set `tab_size` in the language config. Zed falls back to the global `tab_size` setting (`display_map.rs:1375-1377`).
4. **Queries:** `highlights.scm`, `brackets.scm`, `indents.scm`, `outline.scm`, `textobjects.scm`. No `folds.scm` (unsupported); folding comes from indents. No `injections.scm`, `runnables.scm`, or `overrides.scm` in v1.
5. **Delivery:** local dev extension now; structure and metadata must be dev-install-ready. Add a top-level `repository` URL before registry submission; no `zed-industries/extensions` PR in this plan.
6. **Extension ID:** `teal`. Language name: `Teal`. Grammar name: `teal`.
7. **Server lookup:** Zed `LspSettings` binary path, arguments, and env → `worktree.which("teal-language-server")` → LuaRocks user tree (`~/.luarocks/bin` on Unix, `%APPDATA%\luarocks\bin` with a `.bat` wrapper on Windows) → error.
8. **Server arguments:** `-d INFO` by default, so users can inspect server logs. **Never pass `-L`**; log-mode flags crash this server on shutdown (`attempt to use a closed file`, `log_file.lua:65`, exit 1).
9. **License and author:** Apache-2.0; `Ahmad Othman <ahmad.ali.othman@outlook.com>`. `LICENSE` already exists from `license Apache-2.0`.
10. **Verification fixture:** `examples/sample.tl`, covering records, enums, generics, functions, loops, comments, and one deliberate type error plus one unused-variable warning.

## Global Constraints

- Extension ID `teal`; no word `zed` or `extension` in the ID.
- Grammar `rev` must stay `ac6b42ffb23d5d52b582c02508dd3524ac0ba3df`.
- Language config must not contain `tab_size`.
- `teal-language-server` is never bundled or auto-installed.
- LSP id is `teal-language-server`; default args are exactly `["-d", "INFO"]`; never add `-L`.
- Rust crate is a `cdylib` targeting `wasm32-wasip2`, using `zed_extension_api = "0.7.0"`.
- All user-facing text is English.
- No files outside the extension worktree are read or modified by extension code. The LuaRocks user-tree probe goes through `worktree.which`, not `std::fs`.
- Commit messages follow Conventional Commits.

## File Structure

```text
extension.toml
Cargo.toml                       # added in Task 3
src/teal.rs                      # added in Task 3
languages/teal/config.toml
languages/teal/highlights.scm
languages/teal/brackets.scm
languages/teal/indents.scm       # added in Task 2
languages/teal/outline.scm       # added in Task 2
languages/teal/textobjects.scm   # added in Task 2
examples/sample.tl
README.md                        # added in Task 4
LICENSE                          # already generated
.gitignore
```

Responsibilities:
- `extension.toml` — registration only: identity, grammar, and (Task 3) language server.
- `languages/teal/config.toml` — language matching, comments, brackets, indentation fallbacks.
- `*.scm` — one file per Zed query; no query logic shared across files.
- `src/teal.rs` — binary resolution and command construction only.
- `examples/sample.tl` — manual verification fixture; not shipped logic.

## Review Focus

These are the failure modes most likely to bite a user, in order. Each is pinned by a verification step in its owning task.

1. **ABI 15 rejection:** If Zed 1.23.2's tree-sitter runtime rejects the pinned grammar, the dev install fails before any feature works. The upstream repository has no verified ABI-14 tag (`v0.1.0` is ABI 15 and `v0.0.4` is ABI 12), so do not claim a fallback pin; report the Zed version if this happens. Task 1.
2. **LuaRocks bin tree is outside the WASI sandbox:** `std::fs` cannot see it. The fallback must construct the path from `worktree.shell_env()` and resolve it through `worktree.which`. Task 3.
3. **Log-mode shutdown crash:** Any `-L` argument crashes `teal-language-server` 0.2.1 on shutdown. Task 3 code review and runtime log check.
4. **`elseif`/`else` indentation:** tree-sitter's `@indent` alone indents `elseif`/`else` one level too deep; the decrease regex must outdent them. Task 2.
5. **Project root without `tlconfig.lua`:** the server still starts and diagnoses open files, but the verification must prove it rather than assume a config is required. Task 3.

---

### Task 1: Register Teal and core highlighting

**Files:**
- Create: `extension.toml`
- Create: `languages/teal/config.toml`
- Create: `languages/teal/highlights.scm`
- Create: `languages/teal/brackets.scm`
- Create: `examples/sample.tl`
- Delete: `Cargo.toml`, `src/main.rs` (placeholder binary crate; Task 3 recreates `Cargo.toml` as a library)
- Keep: `.gitignore`, `LICENSE`

**Interfaces:**
- Consumes: nothing.
- Produces: extension ID `teal`, language name `Teal`, grammar name `teal`; Task 3 adds the language-server table to this manifest.

- [ ] **Step 1: Write `extension.toml`**

Exact contents:

```toml
id = "teal"
name = "Teal"
description = "Teal language support."
version = "0.1.0"
schema_version = 1
authors = ["Ahmad Othman <ahmad.ali.othman@outlook.com>"]

[grammars.teal]
repository = "https://github.com/euclidianAce/tree-sitter-teal"
rev = "ac6b42ffb23d5d52b582c02508dd3524ac0ba3df"
```

Do not add a `[language_servers...]` table in this task; the extension must remain installable without Rust until Task 3.

- [ ] **Step 2: Write `languages/teal/config.toml`**

Exact keys and values:

```toml
name = "Teal"
grammar = "teal"
path_suffixes = ["tl"]
line_comments = ["-- ", "--- "]
block_comment = ["--[", "]"]
autoclose_before = ";:.,=}])>"
brackets = [
    { start = "{", end = "}", close = true, newline = true },
    { start = "[", end = "]", close = true, newline = true },
    { start = "(", end = ")", close = true, newline = true },
    { start = "\"", end = "\"", close = true, newline = false, not_in = ["string"] },
    { start = "'", end = "'", close = true, newline = false, not_in = ["string"] },
]
collapsed_placeholder = "--[ ... ]--"
```

No `tab_size`, no `roots`, no `[language_servers]` in this file. Those keys do not belong here.

- [ ] **Step 3: Write `languages/teal/highlights.scm`**

First obtain the reference grammar checkout (shallow clone, pinned commit):

```bash
git clone --depth 1 https://github.com/euclidianAce/tree-sitter-teal /tmp/tree-sitter-teal
git -C /tmp/tree-sitter-teal fetch --depth 1 origin ac6b42ffb23d5d52b582c02508dd3524ac0ba3df
git -C /tmp/tree-sitter-teal checkout FETCH_HEAD
```

Use `/tmp/tree-sitter-teal/queries/highlights.scm` as the starting point, then apply this capture remap so every name resolves in Zed themes:

| Grammar/Helix capture | Zed capture |
| --- | --- |
| `@conditional` | `@keyword.control` |
| `@repeat` | `@keyword.control` |
| `@parameter` (value args) | `@variable.parameter` |
| `@parameter` (type args) | `@type.parameter` |
| `@keyword.function` | `@keyword` |
| `@constant.builtin` | keep |
| `@error` | drop |

Keep the grammar's structure for booleans, comments, shebang comments, identifiers, `nil`, numbers, strings, escapes, format specifiers, table-constructor braces, varargs, delimiters, keywords, operators, labels, function statements/calls, record/interface/enum/typedef declarations and bodies, simple/index/union/function types, variable declarators, attributes, and punctuation.

Add these patterns the grammar query does not cover:

- `(table_entry key: (identifier) @property)`
- `(field key: (identifier) @property)`
- `(index key: (identifier) @property)` for dotted access; do not color the `indexed_object` field.
- `(attribute) @attribute`
- `(label) @label`
- `(boolean) @boolean`

Do not capture `method_index` keys as properties; `:` is method-call syntax, and the grammar's function-call patterns already color those names `@function`.

Keep the property captures **before** the function-call captures so a call such as `obj.method()` resolves its name to `@function`, not `@property`. If a call name still renders as a property, remove the conflicting property pattern; do not blindly reorder the whole query.

Use `@variable.special` for `self` (both `(identifier)` and function-signature `arg` forms), matching the grammar query.

- [ ] **Step 4: Write `languages/teal/brackets.scm`**

Exact patterns:

```scheme
("[" @open "]" @close)
("{" @open "}" @close)
("(" @open ")" @close)
```

- [ ] **Step 5: Write `examples/sample.tl`**

Exact contents (validated with `tl check`: one unused-variable warning and one deliberate type error):

```teal
#!/usr/bin/env tl
-- A sample covering the syntax the extension highlights and outlines.

local record Point
  x: number
  y: number
  metamethod __tostring: function(Point): string
end

local enum Color
  "red"
  "green"
  "blue"
end

local type Pair<T> = { T, T }

local function distance(a: Point, b: Point): number
  local dx = a.x - b.x
  local dy = a.y - b.y
  return math.sqrt(dx * dx + dy * dy)
end

local p: Point = { x = 0, y = 0 }
local q: Point = { x = 3, y = 4 }

for i = 1, 3 do
  print(i, distance(p, q))
end

while distance(p, q) > 0 do
  break
end

repeat
  local unused = 1
until true

if distance(p, q) > 3 then
  print("far")
elseif distance(p, q) == 5 then
  print("five")
else
  print("near")
end

-- Deliberate type error for diagnostics.
local label: string = distance(p, q)
local pair: Pair<number> = { 1, 2 }
print(label, "red" as Color, pair[1])
```

- [ ] **Step 6: Remove the placeholder crate**

Delete `Cargo.toml` and `src/main.rs`. The repo must not advertise a Rust build until Task 3. Leave the empty `src/` directory out of git.

- [ ] **Step 7: Install and verify the language extension in Zed**

Run `zed --foreground` to capture logs. In Zed, open Extensions → Install Dev Extension and select `/home/othman/work/Rust/teal-zed-extension`. Then open `examples/sample.tl`.

Expected:
- The status bar language is **Teal**, not Plain Text.
- Keywords, strings, numbers, comments, types, function names, table keys, and labels are colored.
- Typing `{`, `[`, `(`, `"`, or `'` auto-closes; quotes do not auto-close inside a string.
- `Ctrl-/` toggles `-- ` line comments.
- The Zed log shows no tree-sitter grammar or query errors.

If the grammar fails with an ABI/version error, stop and report it. Do not silently downgrade; the fallback decision belongs to the user.

- [ ] **Step 8: Commit**

```bash
git add extension.toml languages/teal/config.toml languages/teal/highlights.scm \
  languages/teal/brackets.scm examples/sample.tl LICENSE .gitignore
git commit -m "feat: register teal language and highlighting"
```

---

### Task 2: Indentation, outline, and text objects

**Files:**
- Create: `languages/teal/indents.scm`
- Create: `languages/teal/outline.scm`
- Create: `languages/teal/textobjects.scm`
- Modify: `languages/teal/config.toml` (add the two fallback regexes)
- Modify: `examples/sample.tl` (add a multiline table and a `do` block)

**Interfaces:**
- Consumes: node names from the pinned grammar; the language name `Teal` from Task 1.
- Produces: indentation, symbol outline, and Vim text objects for Teal.

- [ ] **Step 1: Write `languages/teal/indents.scm`**

Confirm node and field names against `/tmp/tree-sitter-teal/src/node-types.json` from Task 1. Use these exact node/literal pairs:

```scheme
(if_statement "end" @end) @indent
(do_statement "end" @end) @indent
(while_body "end" @end) @indent
(for_body "end" @end) @indent
(repeat_statement "until" @end) @indent
(function_body "end" @end) @indent
(macroexp_body "end" @end) @indent
(record_body "end" @end) @indent
(interface_body "end" @end) @indent
(enum_body "end" @end) @indent
(table_constructor "}" @end) @indent
(_ "(" ")" @end) @indent
(_ "[" "]" @end) @indent
```

Rationale: `while_body` and `for_body` are aliases of `do_statement`; matching `do_statement` would miss them. Function, record, interface, enum, and macroexp bodies own their `end`, so the capture lives on the body node.

Do not add a `@start` capture; Zed's indentation works from the node range and the `@end` marker.

- [ ] **Step 2: Add fallback regexes to `languages/teal/config.toml`**

Append exactly:

```toml
increase_indent_pattern = "^\\s*(if|elseif|else|do|while|for|repeat|((local|global)\\s+)?function|.+=\\s*function|.*[,(]\\s*function|((local|global)\\s+)?(record|interface|enum|macroexp|type))\\b"
decrease_indent_pattern = "^\\s*(elseif|else|end|until)\\b"
```

Expected: pressing Enter after `then`, `do`, `function ...`, `record`, `interface`, `enum`, `repeat`, or an opening brace indents one level; typing `elseif`, `else`, `end`, or `until` outdents. The regexes cover incomplete code before the parser produces a complete node.

- [ ] **Step 3: Write `languages/teal/outline.scm`**

Exact items:

```scheme
(function_statement
  "function" @context
  name: (_) @name) @item

(record_declaration
  "record" @context
  name: (identifier) @name) @item

(interface_declaration
  "interface" @context
  name: (identifier) @name) @item

(enum_declaration
  "enum" @context
  name: (identifier) @name) @item

(type_declaration
  "type" @context
  name: (identifier) @name) @item

(macroexp_declaration
  "macroexp" @context
  name: (identifier) @name) @item

(macroexp_statement
  "macroexp" @context
  name: (identifier) @name) @item
```

Do not capture every `identifier`; that floods the outline and is the defect in the unregistered community extension.

- [ ] **Step 4: Write `languages/teal/textobjects.scm`**

Exact patterns:

```scheme
(function_statement
  body: (function_body
    (_)* @function.inside)) @function.around

(anon_function
  body: (function_body
    (_)* @function.inside)) @function.around

(record_declaration
  record_body: (record_body
    (_)* @class.inside)) @class.around

(anon_record
  record_body: (record_body
    (_)* @class.inside)) @class.around

(interface_declaration
  interface_body: (interface_body
    (_)* @class.inside)) @class.around

(anon_interface
  interface_body: (interface_body
    (_)* @class.inside)) @class.around

(enum_declaration
  enum_body: (enum_body
    (_)* @class.inside)) @class.around

(comment)+ @comment.around
```

Text objects are Vim-mode only in Zed. Capturing `(_)*` rather than the body node keeps the closing `end` out of the inside selection; Zed unions captures of the same object. Do not add a `@comment.inside`; Zed's documented comment capture is `@comment.around`.

- [ ] **Step 5: Extend `examples/sample.tl`**

Add a multiline table, a `do` block, and a top-level `macroexp` declaration so indentation, text objects, and the outline have more than one shape to test:

```teal
do
  local settings = {
    name = "demo",
    values = { 1, 2, 3 },
  }
  print(settings.name)
end
```

Keep the macroexp used (for example `print(double(2))`) so it does not add an unused-function warning, and keep the deliberate type error and unused variable at the end.

- [ ] **Step 6: Verify indentation, outline, and text objects**

Reinstall the dev extension if Zed does not hot-reload it. Open `examples/sample.tl` and check:

- Place the cursor after `if distance(p, q) > 3 then`, press Enter: the new line indents. Type `elseif`: it outdents to the `if` column. Type `else`: same. Type `end`: same.
- Repeat inside `for`, `while`, `repeat`, `function`, `record`, `do`, and the multiline `{` table.
- Open the symbol outline (Go to Symbol): it lists `Point`, `Color`, `Pair`, `double`, `distance`, and `settings` is not listed (a local table is not a declaration). No duplicate entries for `x`/`y`.
- In Vim mode, `vaf` inside `distance` selects the function body; `vac` inside `Point` selects the record body; `vF`/comment text object selects the comment run (if the keymap is configured).
- The Zed log shows no query parse errors for `indents.scm`, `outline.scm`, or `textobjects.scm`.

- [ ] **Step 7: Commit**

```bash
git add languages/teal/indents.scm languages/teal/outline.scm \
  languages/teal/textobjects.scm languages/teal/config.toml examples/sample.tl
git commit -m "feat: add teal indentation outline and text objects"
```

---

### Task 3: Connect `teal-language-server`

**Files:**
- Create: `Cargo.toml`
- Create: `src/teal.rs`
- Modify: `extension.toml` (add the language-server table)
- Modify: `examples/sample.tl` only if the diagnostics fixture needs adjustment; keep the intentional error.

**Interfaces:**
- Consumes: extension id `teal`, language name `Teal`, grammar name `teal`.
- Produces: `TealExtension` implementing `zed::Extension`; server id `teal-language-server`.

- [ ] **Step 1: Write `Cargo.toml`**

Exact contents:

```toml
[package]
name = "teal-zed-extension"
version = "0.1.0"
edition = "2021"
publish = false
license = "Apache-2.0"

[lib]
path = "src/teal.rs"
crate-type = ["cdylib"]

[dependencies]
zed_extension_api = "0.7.0"
```

Do not add other dependencies. The extension API already provides settings, worktree access, and command types.

- [ ] **Step 2: Write `src/teal.rs`**

Required public shape:

```rust
use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result, Worktree};

struct TealExtension;

impl zed::Extension for TealExtension {
    fn new() -> Self;
    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<zed::Command>;
}

zed::register_extension!(TealExtension);
```

Implementation decisions:

- `language_server_id.as_ref()` is the string `"teal-language-server"`.
- Read `LspSettings::for_worktree(...).ok()`; treat missing or invalid settings as empty, not as an error.
- Resolution order: `settings.binary.path` → `worktree.which("teal-language-server")` → the LuaRocks user tree.
- For the LuaRocks fallback, read `shell_env` from `worktree.shell_env()`. On Unix, use `HOME` then `USERPROFILE` and build `{home}/.luarocks/bin/teal-language-server`. On Windows, use `%APPDATA%\luarocks\bin\teal-language-server.bat` first, then `HOME`/`USERPROFILE` with `/.luarocks/bin/teal-language-server.bat` for MSYS2-MinGW installs. Pure MSYS/Cygwin suffixless wrappers are covered by `PATH` or `binary.path`, not this fallback. Do not use `std::env::var` (WASI exposes only `PWD` and `RUST_BACKTRACE`) and do not use `std::fs` (the sandbox does not preopen the home directory). Resolve each candidate with `worktree.which(&candidate)`.
- Arguments: if `settings.binary.arguments` is `Some`, use it verbatim. Otherwise use `["-d", "INFO"]`. Never add `-L`, `--log-mode`, or `--verbose`.
- Environment: pass `settings.binary.env` through to `zed::Command.env`; default to an empty vector when unset.
- If no path resolves, return this error string (use a raw string literal; it contains backticks):
  `teal-language-server was not found. Install it with `luarocks install teal-language-server`, or set `lsp.teal-language-server.binary.path` in Zed settings. See https://github.com/teal-language/teal-language-server.`
- Do not implement `language_server_initialization_options` or `language_server_workspace_configuration`; this server ignores both. Do not implement completion/symbol labels in v1.

- [ ] **Step 3: Add the server table to `extension.toml`**

Append exactly:

```toml
[language_servers.teal-language-server]
name = "Teal Language Server"
languages = ["Teal"]
```

`languages` must match the `name` in `languages/teal/config.toml` exactly (`Teal`, capital T). A mismatch silently disables the server.

- [ ] **Step 4: Build the wasm library**

Run:

```bash
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2
```

Expected: build succeeds and `target/wasm32-wasip2/release/teal_zed_extension.wasm` exists.

- [ ] **Step 5: Verify the LSP in a real project**

Reinstall the dev extension so Zed rebuilds the wasm. Then:

```bash
rm -rf /tmp/teal-demo
mkdir -p /tmp/teal-demo
cp examples/sample.tl /tmp/teal-demo/main.tl
printf 'return { source_dir = "." }\n' > /tmp/teal-demo/tlconfig.lua
zed --foreground /tmp/teal-demo
```

Expected:
- Diagnostics: one error, `in local declaration: label: got number, expected string`; one warning, `unused variable unused: integer`.
- Hover over `distance` shows its function signature (markdown with types).
- Completion after typing `p.x` offers the record fields `x` and `y`. A bare `p.` returns only `(none)` in server 0.2.1; do not treat that upstream limitation as a failure.
- The server log appears on stderr with `INFO ...` lines; no `attempt to use a closed file` and no exit code 1.
- The server starts even though `tlconfig.lua` is present; then repeat the same check without `tlconfig.lua` in a second directory (`/tmp/teal-demo-noconfig`). Diagnostics must still appear. This pins Review Focus item 5.

- [ ] **Step 6: Review the missing-binary error and the luarocks fallback**

The missing-binary branch is the "all three lookups returned `None`" case. A bogus configured `binary.path` is **not** a valid test: it takes the first branch and produces a spawn error from Zed instead. Verify by code review that every `None` path reaches the Step 2 error string.

If a clean environment is available (container or VM with no `teal-language-server` on the shell `PATH` and no LuaRocks user-tree entry), open a `.tl` file there and confirm the message appears. Do not claim a runtime pass on a host where the server is always installed.

Optional runtime check for the Unix LuaRocks fallback: create `~/.luarocks/bin/teal-language-server` as a symlink to the real wrapper, launch Zed with a shell `PATH` that contains `lua5.5` but not `/usr/bin/teal-language-server`, and confirm the server starts. If this is too invasive for the machine, state that the fallback was code-reviewed but not runtime-tested; do not claim it passed. The Windows `%APPDATA%\luarocks\bin\teal-language-server.bat` candidate is covered by path-construction unit tests only unless a Windows host is available.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock src/teal.rs extension.toml
git commit -m "feat: connect teal-language-server"
```

---

### Task 4: Documentation and final verification

**Files:**
- Create: `README.md`
- Verify: `LICENSE`, `extension.toml`, `.gitignore`
- No code changes.

**Interfaces:**
- Consumes: the finished extension from Tasks 1–3.
- Produces: user-facing install, formatter, and troubleshooting documentation.

- [ ] **Step 1: Write `README.md`**

Sections, in this order:

1. **Teal for Zed** — one paragraph: syntax highlighting, brackets, indentation, outline, text objects, and `teal-language-server` integration.
2. **Requirements** — `teal-language-server` (install with `luarocks install teal-language-server`); `tl` only if the user wants to run Teal.
3. **Install** — local dev install: Extensions → Install Dev Extension → select this repository. Note that registry publishing comes later.
4. **Language server** — resolution order, the `lsp.teal-language-server.binary.path` / `arguments` settings example, and the default `-d INFO`. State that `-L`/`--log-mode` is not passed because 0.2.1 crashes on shutdown with it.
5. **Formatter** — Zed does not ship a Teal formatter in this extension. Document the external formatter:

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

   Use 2 spaces to match the author's Helix setup; mention that upstream Teal uses 3 and that the value is a user preference. `ceru` reads stdin and writes stdout, which is what Zed expects.
6. **What's included** — the five query files, the pinned grammar, and `examples/sample.tl` as the manual fixture.
7. **Troubleshooting** — server not found; no diagnostics (check `tlconfig.lua` and the server log); grammar ABI mismatch (no verified fallback pin; report the Zed version).
8. **License** — Apache-2.0, link `LICENSE`.

Keep it concise; no marketing copy.

- [ ] **Step 2: Verify metadata and license**

Run:

```bash
head -5 LICENSE
grep -n "id\|authors\|grammars\|language_servers" extension.toml
git status --short
```

Expected: the Apache-2.0 header; `id = "teal"`, the author line, the grammar table, and the server table; no unexpected tracked changes.

Do not regenerate `LICENSE` unless it is missing or not Apache-2.0.

- [ ] **Step 3: Final end-to-end check**

Remove the dev extension and install it again from `/home/othman/work/Rust/teal-zed-extension`. Open `examples/sample.tl` and confirm:

- Language is Teal; highlighting covers every construct in the fixture.
- Brackets and comment toggling work.
- Newlines indent and `elseif`/`else`/`end` outdent.
- Outline lists `Point`, `Color`, `Pair`, `double`, and `distance`.
- Diagnostics show the deliberate type error and the unused-variable warning.
- Hover and completion work.
- `zed --foreground` logs contain no query errors and no server shutdown crash.

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs: document teal extension usage"
```

---

## Verification Summary

| Feature | Command / action | Expected |
| --- | --- | --- |
| Grammar builds | Install dev extension | No tree-sitter ABI/query error |
| Language detection | Open `examples/sample.tl` | Status bar says Teal |
| Highlights | Inspect fixture | Keywords, types, strings, properties, labels colored |
| Brackets/comments | Type brackets; `Ctrl-/` | Auto-close; `-- ` toggle |
| Indentation | Enter after `then`/`do`/`{`; type `elseif`/`end` | Indent/outdent correctly |
| Outline | Go to Symbol | `Point`, `Color`, `Pair`, `double`, `distance` |
| Text objects | Vim `vaf`/`vac` | Function/record body selected |
| LSP diagnostics | Open `/tmp/teal-demo/main.tl` | 1 error + 1 warning |
| LSP hover/completion | Hover `distance`; type `p.x` | Signature; `x`, `y` |
| Missing server | Code review; clean environment if available | Actionable error |
| No `tlconfig.lua` | Open `/tmp/teal-demo-noconfig` | Diagnostics still work |
| Formatter docs | Read README | `ceru` config uses stdin/stdout |
| License | `head -5 LICENSE` | Apache-2.0 |

## Open Questions

None. All decisions are settled. Automatic `teal-language-server` installation and registry publication are explicitly out of scope; both can be planned later without changing this architecture.
