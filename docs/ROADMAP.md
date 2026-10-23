# Roadmap — deferred and future work

v1's scope was frozen at planning time (`docs/plans/2026-10-22-teal-zed-extension.md`):
language support plus the LSP integration, shipped as a dev extension. Two items were
explicitly deferred ("both can be planned later without changing this architecture",
plan line 649). This file is the standing record so none of it lives only in session
history.

Recorded 2026-10-23. Origin: <https://github.com/AhmedOsman101/teal-zed-extension>.

## 1. Registry publication — ready to execute

Deferred in the plan (Delivery decision, line 21): "Add a top-level `repository` URL
before registry submission; no `zed-industries/extensions` PR in this plan." The URL no
longer needs to be invented — the public repo exists.

- [x] Add `repository = "https://github.com/AhmedOsman101/teal-zed-extension"` to
      `extension.toml` (top-level key, same convention as the grammar entry). Done in
      `c38c4ea`.
- [x] Confirm no `teal` extension id (and no competing Teal grammar) is already claimed
      in `zed-industries/extensions`.
- [ ] Dev-install once more after the manifest change; manifest edits rebuild
      `extension.wasm`.
- [ ] Open the registry PR against `zed-industries/extensions`, referencing this repo.
      Prepared locally: branch `add-teal`, commit `6e2a77c` (`.gitmodules` +
      `[teal]` entry between `tcl` and `tech49-theme`) in
      `.tmp/opencode/extensions`. NOTE: zed-industries/extensions' AI policy
      forbids agent-driven contributions and agent-written PR comments, so the push
      and PR body must come from the repo maintainer in their own words.
- [x] Display name updated to "Teal language support" (`1bff958`); extension id
      stays `teal`.

## 2. Automatic `teal-language-server` installation

Deferred in the plan (Scope decision, line 17): v1 ships an actionable error instead of
installing the server. Not decided yet; candidate directions when revisited:

- Better cross-platform install documentation (Windows especially).
- A user-run installer script in this repo (`pacman`/`apt` package is not the extension's
  job; LuaRocks is).
- Zed's extension sandbox constrains what code a running extension can download or
  execute, so any "auto-install" is realistically docs-first. No architecture change
  required either way (plan line 649).

## 3. Upstream `teal-language-server` gaps

Fixes we shipped upstream: PR
[teal-language/teal-language-server#90](https://github.com/teal-language/teal-language-server/pull/90)
(branch `fix/honor-invoked-completion`, base `upstream/main` @ `ec1aaa3`, two commits):

- Honors `triggerKind == Invoked` completions (guard #1 previously bailed on any
  non-`TriggerCharacter` context, i.e. every Zed request not triggered by `.`/`:`).
- Suppresses definition-position popups only when completion was not explicitly invoked.
- Bare lines and partial field names inside `{ }` resolve the annotation's record fields.
- Forces a type check on parse-broken documents so `.` completion works mid-edit.

Not merged at recording time; server 0.2.1 lacks these behaviors (notably bare-word
completion).

Follow-up issues filed 2026-10-23:

- [#91](https://github.com/teal-language/teal-language-server/issues/91) — completion
  items carry no type information (no `detail`/`documentation`).
- [#92](https://github.com/teal-language/teal-language-server/issues/92) — hovering a
  `macroexp` errors with a nil index in `get_function_args_string`.
- [#93](https://github.com/teal-language/teal-language-server/issues/93) — completion
  after `.` returns nothing on parse-incomplete files (unclosed table constructor).
- Type-name completion in type positions: already tracked as upstream
  [#30](https://github.com/teal-language/teal-language-server/issues/30); concrete
  repro and item-source analysis added there as a comment (no duplicate issue filed).

Still missing upstream — candidates for follow-up issues:

- Completion items carry labels only: no `detail`/`documentation`, so no type info in
  the popup even though the server computes it internally.
- No type-name completion in type positions (`local p: Poin<TAB>`).
- Hover over a `macroexp` crashes the server internally (`get_function_args_string`
  nil-index).

## 4. Untested environments

Residuals carried from the re-review (ledger, "Residual uncertainty") — extend here as
they get tested:

- [ ] Windows: the LuaRocks fallback paths (`%APPDATA%\luarocks\bin\`, then
      `~/.luarocks\bin\`) are unit-tested only; no Windows host has run the extension.
- [ ] Zed GUI: text objects, outline, auto-close await manual verification (diagnostics,
      hover, dot-triggered completion, and dev-install were verified live 2026-10-23).
- [ ] ABI-15 grammar acceptance on Zed versions other than 1.23.2.

## 5. Repository housekeeping

- [ ] Merge `feat/teal-zed-extension` into `main` and set the GitHub default branch
      (the repo was published from the feature branch; local `main` was never created —
      the repo started with an unborn HEAD).
- [ ] Decide on `.zed/settings.json` (untracked, machine-specific binary path + `-d
      TRACE` logging): gitignore it or commit a shared example.
- [ ] `examples/sample.tl` has an uncommitted local edit (`label: string` → `number`,
      line 60) that removes the planted type error; the verification baseline (`tl
      check` = exactly one warning and one error) depends on that error. Keep the edit
      and relax the baseline, or restore the error.

## 6. Extension follow-ups when upstream #90 merges

- [ ] Re-run the headless LSP smoke against the first server release containing the fix
      so README statements match reality.
- [ ] Re-check README's completion troubleshooting (the bare-`.` note and the new
      bare-word behavior) against the new server.
