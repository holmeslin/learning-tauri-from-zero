//! 驗證書中的程式碼。一集（一個 .md 檔）是一個專案的時間線，由上往下重播：
//!
//! - 「`路徑`：」加程式區塊：專案裡這個檔案現在的內容，路徑從專案根目錄算起
//!   （`src-tauri/src/lib.rs`、`src/main.ts`）。同一路徑再出現就覆蓋。
//!   沒寫到的檔案沿用 `template/`（`create-tauri-app` 的 vanilla-ts 範本）。
//! - 「`npm run tauri dev` 的輸出：」這類標籤加 `text` 區塊：檢查點。這一刻把檔案寫進範本，
//!   執行 `cargo fmt --check`、`cargo clippy -D warnings`、`tsc --noEmit`；
//!   書上貼了 `error…` 就必須失敗（錯誤碼也要對），否則必須成功。輸出文字不逐字比對。
//! - 最後一個檢查點之後還改過檔案，檔案結尾視為一個「必須成功」的檢查點。
//! - 沒有路徑標籤的 rust / ts / js / json / toml 區塊必須加 `ignore`，代表刻意不驗證的片段。
//!
//! 規則全文見 `docs/style-guide.md` 的「程式碼區塊」。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// 會被驗證的檢查點指令（`npm install`、`npm create` 這類不編譯的指令不驗證）。
const CHECKED_COMMANDS: &[&str] = &[
    "npm run tauri dev",
    "npm run tauri build",
    "npm run build",
    "cargo check",
    "cargo build",
    "cargo clippy",
    "npx tsc",
];

/// 沒有路徑標籤時必須加 `ignore` 的語言。
const VERIFIABLE_LANGS: &[&str] = &[
    "rust",
    "ts",
    "typescript",
    "js",
    "javascript",
    "json",
    "toml",
];

#[derive(Debug, PartialEq)]
enum Label {
    File(String),
    Out(String),
}

#[derive(Debug)]
enum Step {
    File {
        path: String,
        code: String,
        line: usize,
    },
    Check {
        cmd: String,
        shown: String,
        line: usize,
    },
}

/// 路徑：只有英數與 `_ . / -`，不能是絕對路徑或含 `..`，檔名要有副檔名。
fn valid_path(path: &str) -> bool {
    path.chars()
        .all(|c| c.is_ascii_alphanumeric() || "_./-".contains(c))
        && !path.starts_with('/')
        && path.split('/').all(|part| part != "..")
        && path
            .rsplit('/')
            .next()
            .and_then(|name| name.rsplit_once('.'))
            .is_some_and(|(_, ext)| {
                !ext.is_empty() && ext.chars().all(|c| c.is_ascii_alphanumeric())
            })
}

/// 標籤整行只有一個行內程式碼加全形冒號，句尾可以多一段（說明）。
fn parse_label(line: &str) -> Option<Label> {
    let line = line.strip_suffix('：')?;
    let line = match line.strip_suffix('）') {
        Some(rest) => rest.rsplit_once('（')?.0,
        None => line,
    };
    if let Some(cmd) = line
        .strip_suffix("` 的輸出")
        .and_then(|rest| rest.strip_prefix('`'))
    {
        return Some(Label::Out(cmd.to_string()));
    }
    let path = line.strip_prefix('`')?.strip_suffix('`')?;
    valid_path(path).then(|| Label::File(path.to_string()))
}

/// 圍欄開頭：(反引號數, info string)。
fn fence_open(raw: &str) -> Option<(usize, &str)> {
    let text = raw.trim_start();
    let n = text.len() - text.trim_start_matches('`').len();
    (n >= 3).then(|| (n, text[n..].trim()))
}

fn fence_close(raw: &str, n: usize) -> bool {
    let text = raw.trim();
    text.len() >= n && text.chars().all(|c| c == '`')
}

/// 掃出一集的時間線，以及沒標路徑又沒加 `ignore` 的區塊（行號）。
fn extract(text: &str) -> (Vec<Step>, Vec<usize>) {
    let mut steps = Vec::new();
    let mut unlabeled = Vec::new();
    let mut label = None;
    let mut lines = text.lines().enumerate();
    while let Some((i, raw)) = lines.next() {
        if let Some((n, info)) = fence_open(raw) {
            let mut code = String::new();
            for (_, body) in lines.by_ref() {
                if fence_close(body, n) {
                    break;
                }
                code.push_str(body);
                code.push('\n');
            }
            let mut opts = info.split(',').map(str::trim);
            let lang = opts.next().unwrap_or_default();
            let ignored = opts.any(|o| o == "ignore");
            let line = i + 1;
            match label.take() {
                Some(Label::File(path)) => steps.push(Step::File { path, code, line }),
                Some(Label::Out(cmd)) if CHECKED_COMMANDS.iter().any(|c| cmd.starts_with(c)) => {
                    steps.push(Step::Check {
                        cmd,
                        shown: code,
                        line,
                    })
                }
                Some(Label::Out(_)) => {}
                None if VERIFIABLE_LANGS.contains(&lang) && !ignored => unlabeled.push(line),
                None => {}
            }
        } else if let Some(l) = parse_label(raw.trim()) {
            label = Some(l);
        } else if !raw.trim().is_empty() {
            label = None;
        }
    }
    (steps, unlabeled)
}

/// 書上貼的錯誤碼：Rust 的 `error[E0308]` 與 TypeScript 的 `error TS2345`。
fn error_codes(text: &str) -> Vec<String> {
    let rust = text.match_indices("error[").filter_map(|(i, _)| {
        text[i + 6..]
            .split_once(']')
            .map(|(code, _)| code.to_string())
    });
    let ts = text.match_indices("error TS").map(|(i, _)| {
        let digits: String = text[i + 8..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        format!("TS{digits}")
    });
    rust.chain(ts).collect()
}

fn shows_error(text: &str) -> bool {
    text.lines().any(|l| {
        let l = l.trim_start();
        l.starts_with("error") || l.contains("error TS")
    })
}

struct Env {
    template: PathBuf,
    work: PathBuf,
    target: PathBuf,
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name();
        // node_modules 改用 symlink；target、gen 是建置產物。
        if ["node_modules", "target", "gen", "dist"].contains(&name.to_str().unwrap_or_default()) {
            continue;
        }
        let dest = to.join(&name);
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}

/// 預設用 `--locked`；一集貼了 `src-tauri/Cargo.toml` 就不加。
/// 理由見 `docs/style-guide.md` 的「程式碼區塊」。
fn use_locked(files: &BTreeMap<String, String>) -> bool {
    !files.contains_key("src-tauri/Cargo.toml")
}

fn run(cmd: &mut Command) -> Result<(bool, String), String> {
    let out = cmd.output().map_err(|e| format!("無法執行 {cmd:?}：{e}"))?;
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    Ok((out.status.success(), text))
}

impl Env {
    fn write_project(&self, files: &BTreeMap<String, String>) -> Result<(), String> {
        let io = |e: std::io::Error| e.to_string();
        let _ = fs::remove_dir_all(&self.work);
        copy_dir(&self.template, &self.work).map_err(io)?;
        let modules = self.template.join("node_modules");
        if !modules.is_dir() {
            return Err(format!(
                "找不到 {}，請先執行 `npm ci --prefix tools/checkcode/template`",
                modules.display()
            ));
        }
        symlink_dir(&modules, &self.work.join("node_modules")).map_err(io)?;
        for (path, code) in files {
            let full = self.work.join(path);
            fs::create_dir_all(full.parent().unwrap_or(&self.work)).map_err(io)?;
            fs::write(full, code).map_err(io)?;
        }
        Ok(())
    }

    /// 回傳 (排版問題, 編譯是否成功, 編譯輸出)。
    fn check(
        &self,
        files: &BTreeMap<String, String>,
    ) -> Result<(Option<String>, bool, String), String> {
        self.write_project(files)?;
        let tauri = self.work.join("src-tauri");
        let (fmt_ok, fmt_text) = run(Command::new("cargo")
            .args(["fmt", "--check"])
            .current_dir(&tauri))?;
        let mut clippy = Command::new("cargo");
        clippy.arg("clippy");
        if use_locked(files) {
            clippy.arg("--locked");
        }
        let (rust_ok, rust_text) = run(clippy
            .args(["--all-targets", "--", "-D", "warnings"])
            .current_dir(&tauri)
            .env("CARGO_TARGET_DIR", &self.target)
            .env("CARGO_TERM_COLOR", "never"))?;
        let (ts_ok, ts_text) = run(Command::new(self.work.join("node_modules/.bin/tsc"))
            .args(["--noEmit", "--pretty", "false"])
            .current_dir(&self.work))?;
        let fmt = (!fmt_ok).then(|| format!("rustfmt 排版不符：\n{fmt_text}"));
        Ok((fmt, rust_ok && ts_ok, format!("{rust_text}{ts_text}")))
    }
}

#[cfg(unix)]
fn symlink_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(from, to)
}

#[cfg(windows)]
fn symlink_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(from, to)
}

/// 一個檢查點的結果；`Err` 是要回報的原因。
fn checkpoint(
    env: &Env,
    files: &BTreeMap<String, String>,
    cmd: &str,
    shown: &str,
) -> Result<(), String> {
    let (fmt, ok, text) = env.check(files)?;
    if let Some(msg) = fmt {
        return Err(msg);
    }
    match (shows_error(shown), ok) {
        (false, true) => Ok(()),
        (false, false) => Err(format!("`{cmd}` 書上是成功，實際卻失敗：\n{text}")),
        (true, true) => Err(format!("`{cmd}` 書上貼的是錯誤，實際卻成功")),
        (true, false) => match error_codes(shown)
            .into_iter()
            .find(|code| !text.contains(code.as_str()))
        {
            None => Ok(()),
            Some(code) => Err(format!(
                "`{cmd}` 書上的錯誤碼 {code} 實際沒有出現：\n{text}"
            )),
        },
    }
}

/// 重播一集，回傳 (行號, 原因)。
fn replay(env: &Env, steps: &[Step]) -> Vec<(usize, String)> {
    let mut bad = Vec::new();
    let mut files = BTreeMap::new();
    let mut pending = None;
    for step in steps {
        match step {
            Step::File { path, code, line } => {
                files.insert(path.clone(), code.clone());
                pending = Some(*line);
            }
            Step::Check { cmd, shown, line } => {
                pending = None;
                if let Err(msg) = checkpoint(env, &files, cmd, shown) {
                    bad.push((*line, msg));
                }
            }
        }
    }
    if let Some(line) = pending
        && let Err(msg) = checkpoint(env, &files, "（檔案結尾）", "")
    {
        bad.push((line, msg));
    }
    bad
}

fn markdown_files(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            markdown_files(&entry?.path(), out)?;
        }
    } else if path.extension().is_some_and(|e| e == "md") {
        out.push(path.to_path_buf());
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("用法：checkcode <目錄或 .md 檔>...");
        return ExitCode::from(2);
    }
    let mut paths = Vec::new();
    for arg in &args {
        if let Err(e) = markdown_files(Path::new(arg), &mut paths) {
            eprintln!("{arg}：{e}");
            return ExitCode::from(2);
        }
    }
    paths.sort();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let template = root.join("template");
    // 放在 repo 裡（不放系統暫存目錄），根目錄的 rust-toolchain.toml 才會生效。
    let env = Env {
        work: root.join("work"),
        target: template.join("src-tauri/target"),
        template,
    };

    let mut failures = 0;
    let mut projects = 0;
    for path in &paths {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("{}：{e}", path.display());
                return ExitCode::from(2);
            }
        };
        let (steps, unlabeled) = extract(&text);
        let mut bad: Vec<(usize, String)> = unlabeled
            .into_iter()
            .map(|line| {
                (
                    line,
                    "沒有路徑標籤的程式區塊要加 `ignore`（或在前一行補上路徑標籤）".to_string(),
                )
            })
            .collect();
        if !steps.is_empty() {
            projects += 1;
            bad.extend(replay(&env, &steps));
        }
        for (line, msg) in bad {
            failures += 1;
            eprintln!("{}:{line}: {msg}", path.display());
        }
    }
    let _ = fs::remove_dir_all(&env.work);
    println!(
        "檢查了 {} 個檔案、{projects} 個專案，{failures} 個問題",
        paths.len()
    );
    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels() {
        assert_eq!(
            parse_label("`src-tauri/src/lib.rs`："),
            Some(Label::File("src-tauri/src/lib.rs".into()))
        );
        assert_eq!(
            parse_label("`src/main.ts`（加上按鈕）："),
            Some(Label::File("src/main.ts".into()))
        );
        assert_eq!(
            parse_label("`npm run tauri dev` 的輸出（省略編譯進度）："),
            Some(Label::Out("npm run tauri dev".into()))
        );
        assert_eq!(parse_label("`../secret.rs`："), None);
        assert_eq!(parse_label("看看 `src/main.ts`："), None);
    }

    #[test]
    fn timeline() {
        let md = "`src/main.ts`：\n\n```ts\nlet a = 1;\n```\n\n```ts\nfragment\n```\n\n```ts,ignore\nok\n```\n\n`npm install` 的輸出：\n\n```text\nadded\n```\n\n`npm run tauri dev` 的輸出：\n\n```text\nerror[E0308]: mismatched types\n```\n";
        let (steps, unlabeled) = extract(md);
        assert_eq!(unlabeled, vec![7]);
        assert_eq!(steps.len(), 2);
        assert!(matches!(&steps[0], Step::File { path, .. } if path == "src/main.ts"));
        assert!(matches!(&steps[1], Step::Check { cmd, .. } if cmd == "npm run tauri dev"));
    }

    #[test]
    fn codes() {
        let text = "error[E0308]: mismatched types\nsrc/main.ts(3,1): error TS2345: bad\n";
        assert_eq!(error_codes(text), vec!["E0308", "TS2345"]);
        assert!(shows_error(text));
        assert!(!shows_error("Running `target/debug/tauri-app`\n"));
    }

    #[test]
    fn locked_unless_cargo_toml_replaced() {
        let mut files = BTreeMap::new();
        files.insert("src-tauri/src/lib.rs".to_string(), String::new());
        assert!(use_locked(&files), "沒貼 Cargo.toml 時要加 --locked");

        files.insert("src-tauri/tauri.conf.json".to_string(), String::new());
        assert!(use_locked(&files), "貼了別的設定檔不影響，仍要加 --locked");

        // 只看有沒有貼這個檔，不看內容（devtools、isolation 都一樣）。
        files.insert(
            "src-tauri/Cargo.toml".to_string(),
            "features = [\"devtools\"]".to_string(),
        );
        assert!(
            !use_locked(&files),
            "貼了 src-tauri/Cargo.toml 就不加 --locked"
        );
    }
}
