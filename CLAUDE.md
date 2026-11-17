## Writing the book

Before writing or editing anything under `zh-TW/src/`, read `docs/style-guide.md` (rules and the code-verification conventions), `docs/outline.md` (chapter plan and per-chapter boundaries) and `GLOSSARY.md` (terms). Verify examples with `cargo run --manifest-path tools/checkcode/Cargo.toml -- zh-TW/src`.

## Agent skills

### Issue tracker

Issues live in GitHub Issues for `holmeslin/learning-tauri-from-zero`, managed via the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
