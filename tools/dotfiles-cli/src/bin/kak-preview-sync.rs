use dotfiles_cli::{
    command_output, ensure_dir, home_dir, kak_quote_single, sanitize_scope, state_root,
    toml_escape, unique_suffix, user_name,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

struct State {
    scope: String,
    path: PathBuf,
    client: String,
    path_file: PathBuf,
    session_file: PathBuf,
    semantic_file: PathBuf,
    log_file: PathBuf,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.get(1).map(String::as_str) == Some("--semantic-refresh") {
        run_semantic_refresh(&args);
        return;
    }

    let background_run = args.get(1).map(String::as_str) == Some("--background-run");
    let path_arg_index = if background_run { 2 } else { 1 };

    if !background_run && env::var("KAK_PREVIEW_SYNC_FOREGROUND").is_err() {
        spawn_background_run(&args);
        return;
    }

    let Some(path_arg) = args.get(path_arg_index) else {
        return;
    };
    let path = PathBuf::from(path_arg);
    if !path.is_file() {
        return;
    }
    let path = path.canonicalize().unwrap_or(path);

    let state = build_state(path);
    if is_same_last_path(&state) {
        return;
    }
    let _ = fs::write(&state.path_file, state.path.to_string_lossy().as_bytes());

    let Some(session) = resolve_session(&state) else {
        log_warn(&state, None, "no live Kakoune session resolved");
        return;
    };

    let lsp_servers_cmd = ensure_compile_commands_cache(&state.path)
        .map(|(project_root, compile_commands_dir)| {
            let project_root = toml_escape(&project_root.to_string_lossy());
            let compile_commands_dir = toml_escape(&compile_commands_dir.to_string_lossy());
            format!(
                r#"try %{{ set-option buffer lsp_servers %{{
    [clangd]
    args = ["--log=error", "--background-index", "--compile-commands-dir={compile_commands_dir}"]
    offset_encoding = "utf-8"
    root = "{project_root}"
}} }}
try %{{ lsp-did-change-config }}"#
            )
        })
        .unwrap_or_default();

    let preview_cmd = format!(
        "edit -existing -- '{}'\n{}",
        kak_quote_single(&state.path.to_string_lossy()),
        lsp_servers_cmd
    );
    let cmd = eval_client_command(&state.client, &preview_cmd);

    match send_to_kak(&session, &cmd) {
        Ok(()) => schedule_semantic_refresh(&state, &session),
        Err(message) => log_warn(&state, Some(&session), &format!("kak -p failed: {message}")),
    }
}

fn spawn_background_run(args: &[String]) {
    let Some(path_arg) = args.get(1) else {
        return;
    };
    let Ok(exe) = env::current_exe() else {
        return;
    };

    let _ = Command::new(exe)
        .arg("--background-run")
        .arg(path_arg)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn build_state(path: PathBuf) -> State {
    let client = env::var("KAK_CLIENT").unwrap_or_else(|_| "preview".to_string());
    let scope = resolve_scope();
    let state_root = state_root();
    let user = user_name();

    State {
        scope: scope.clone(),
        path,
        client,
        path_file: state_root.join(format!("kak-preview-sync-{user}-{scope}.last")),
        session_file: state_root.join(format!("kak-preview-sync-{user}-{scope}.session")),
        semantic_file: state_root.join(format!("kak-preview-sync-{user}-{scope}.semantic")),
        log_file: state_root.join(format!("kak-preview-sync-{user}-{scope}.log")),
    }
}

fn resolve_scope() -> String {
    if let Ok(scope) = env::var("KAK_SYNC_SCOPE") {
        if !scope.is_empty() {
            return sanitize_scope(&scope);
        }
    }

    if env::var("TMUX").is_ok() {
        if let Some(session) = command_output("tmux", ["display-message", "-p", "#S"]) {
            return sanitize_scope(&session);
        }
    }

    "global".to_string()
}

fn is_same_last_path(state: &State) -> bool {
    fs::read_to_string(&state.path_file)
        .map(|last| last == state.path.to_string_lossy())
        .unwrap_or(false)
}

fn log_warn(state: &State, session: Option<&str>, message: &str) {
    let socket_root = env::var("XDG_RUNTIME_DIR")
        .or_else(|_| env::var("TMPDIR"))
        .unwrap_or_else(|_| "/tmp".to_string());
    let tmux_session = command_output("tmux", ["display-message", "-p", "#S"])
        .unwrap_or_else(|| "<unset>".to_string());
    let timestamp = command_output("date", ["+%Y-%m-%d %H:%M:%S %z"]).unwrap_or_default();
    let session = session.unwrap_or("<unset>");
    let text = format!(
        "[{timestamp}] {message}\n  scope={}\n  path={}\n  client={}\n  session={session}\n  tmux_session={tmux_session}\n  TMPDIR={}\n  XDG_RUNTIME_DIR={}\n  socket={}/kakoune-{}/{}\n\n",
        state.scope,
        state.path.display(),
        state.client,
        env::var("TMPDIR").unwrap_or_else(|_| "<unset>".to_string()),
        env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "<unset>".to_string()),
        socket_root,
        user_name(),
        session
    );

    if let Some(parent) = state.log_file.parent() {
        let _ = ensure_dir(parent);
    }
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&state.log_file)
        .and_then(|mut file| file.write_all(text.as_bytes()));
}

fn list_live_sessions() -> Vec<String> {
    command_output("kak", ["-l"])
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.ends_with(" (dead)"))
        .map(ToOwned::to_owned)
        .collect()
}

fn is_live_session(session: &str) -> bool {
    !session.is_empty() && list_live_sessions().iter().any(|live| live == session)
}

fn extract_session_from_text(text: &str) -> Option<String> {
    text.lines().rev().find_map(|line| {
        let marker = "@[";
        let start = line.rfind(marker)? + marker.len();
        let end = line[start..].find(']')? + start;
        let candidate = &line[start..end];
        (!candidate.is_empty()).then(|| candidate.to_string())
    })
}

fn resolve_from_tmux() -> Option<String> {
    if env::var("TMUX").is_err() {
        return None;
    }

    let panes = command_output(
        "tmux",
        [
            "list-panes",
            "-F",
            "#{pane_id}|#{pane_current_command}|#{pane_title}",
        ],
    )?;
    for line in panes.lines() {
        let mut parts = line.splitn(3, '|');
        let pane_id = parts.next().unwrap_or_default();
        let pane_cmd = parts.next().unwrap_or_default();
        let pane_title = parts.next().unwrap_or_default();
        if pane_cmd != "kak" {
            continue;
        }

        if let Some(candidate) = extract_session_from_text(pane_title) {
            if is_live_session(&candidate) {
                return Some(candidate);
            }
        }

        let captured = command_output("tmux", ["capture-pane", "-p", "-t", pane_id, "-S", "-3"])
            .unwrap_or_default();
        if let Some(candidate) = extract_session_from_text(&captured) {
            if is_live_session(&candidate) {
                return Some(candidate);
            }
        }
    }

    None
}

fn pick_single_live_session() -> Option<String> {
    let live = list_live_sessions();
    (live.len() == 1).then(|| live[0].clone())
}

fn resolve_session(state: &State) -> Option<String> {
    if let Ok(session) = env::var("KAK_SESSION") {
        if !session.is_empty() && session != "auto" {
            return Some(session);
        }
    }

    if let Some(session) = resolve_from_tmux() {
        let _ = fs::write(&state.session_file, &session);
        return Some(session);
    }

    if let Ok(cached) = fs::read_to_string(&state.session_file) {
        let cached = cached.trim();
        if is_live_session(cached) {
            return Some(cached.to_string());
        }
    }

    if let Some(session) = pick_single_live_session() {
        let _ = fs::write(&state.session_file, &session);
        return Some(session);
    }

    None
}

fn is_c_family(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("c" | "h" | "cc" | "hh" | "cpp" | "hpp" | "cxx" | "hxx")
    )
}

fn find_upward_file(start: &Path, name: &str) -> Option<PathBuf> {
    let mut current = start;
    loop {
        if current.join(name).is_file() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

fn find_project_root(file: &Path) -> Option<PathBuf> {
    let dir = file.parent()?;
    find_upward_file(dir, "compile_commands.json")
        .or_else(|| find_upward_file(dir, "Makefile"))
        .or_else(|| {
            command_output(
                "git",
                ["-C", &dir.to_string_lossy(), "rev-parse", "--show-toplevel"],
            )
            .map(PathBuf::from)
        })
        .or_else(|| Some(dir.to_path_buf()))
}

fn compile_db_has_file(db: &Path, file: &Path) -> bool {
    let Ok(text) = fs::read_to_string(db) else {
        return false;
    };
    let Ok(Value::Array(entries)) = serde_json::from_str::<Value>(&text) else {
        return false;
    };
    let file = file.to_string_lossy();
    entries.iter().any(|entry| {
        entry_abs_file(entry)
            .map(|candidate| candidate == file)
            .unwrap_or(false)
    })
}

fn entry_abs_file(entry: &Value) -> Option<String> {
    let object = entry.as_object()?;
    let file = object.get("file")?.as_str()?;
    if Path::new(file).is_absolute() {
        return Some(file.to_string());
    }
    let directory = object.get("directory")?.as_str()?;
    Some(
        Path::new(directory)
            .join(file)
            .to_string_lossy()
            .to_string(),
    )
}

fn ensure_compile_commands_cache(file: &Path) -> Option<(PathBuf, PathBuf)> {
    if !is_c_family(file) {
        return None;
    }

    let root = find_project_root(file)?;
    let cache_base = env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join(".cache"))
        .join("kak-preview-sync/clangd");
    let cache_dir = cache_base.join(format!(
        "{}-{:016x}",
        root.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project"),
        fnv1a(root.to_string_lossy().as_bytes())
    ));
    let out = cache_dir.join("compile_commands.json");
    ensure_dir(&cache_dir).ok()?;

    if out.is_file() && root_db_is_not_newer(&out, &root) {
        if compile_db_has_file(&out, file) || is_header(file) {
            return Some((root, cache_dir));
        }
    }

    let mut base = read_compile_commands(&root.join("compile_commands.json"));
    let mut existing: HashSet<String> = base.iter().filter_map(entry_abs_file).collect();

    for source in collect_c_sources(&root) {
        let abs = source.canonicalize().unwrap_or(source);
        let abs_text = abs.to_string_lossy().to_string();
        if !existing.insert(abs_text.clone()) {
            continue;
        }
        base.push(json!({
            "directory": root.to_string_lossy(),
            "file": abs_text,
            "arguments": ["cc", "-Wall", "-Wextra", "-Werror", "-I.", "-Ilibft", "-c", abs.to_string_lossy(), "-o", "/dev/null"],
            "output": "/dev/null"
        }));
    }

    let tmp = cache_dir.join(format!("compile_commands.{}.json", unique_suffix()));
    let text = serde_json::to_string_pretty(&base).ok()?;
    fs::write(&tmp, text).ok()?;
    fs::rename(&tmp, &out).ok()?;
    Some((root, cache_dir))
}

fn root_db_is_not_newer(out: &Path, root: &Path) -> bool {
    let root_db = root.join("compile_commands.json");
    if !root_db.is_file() {
        return true;
    }
    let Ok(out_time) = fs::metadata(out).and_then(|metadata| metadata.modified()) else {
        return false;
    };
    let Ok(root_time) = fs::metadata(root_db).and_then(|metadata| metadata.modified()) else {
        return false;
    };
    out_time >= root_time
}

fn is_header(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("h" | "hh" | "hpp" | "hxx")
    )
}

fn read_compile_commands(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
}

fn collect_c_sources(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_c_sources_inner(root, &mut out);
    out
}

fn collect_c_sources_inner(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if matches!(
                name.as_ref(),
                ".git" | ".cache" | "build" | "cmake-build-debug" | "cmake-build-release"
            ) {
                continue;
            }
            collect_c_sources_inner(&path, out);
        } else if file_type.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("c")
        {
            out.push(path);
        }
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn eval_client_command(client: &str, body: &str) -> String {
    format!("try %{{ eval -client {client} %{{ {body} }} }} catch %{{ eval -client client0 %{{ {body} }} }}")
}

fn send_to_kak(session: &str, cmd: &str) -> Result<(), String> {
    let mut child = Command::new("kak")
        .arg("-p")
        .arg(session)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| err.to_string())?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(format!("{cmd}\n").as_bytes())
            .map_err(|err| err.to_string())?;
    }

    let output = child.wait_with_output().map_err(|err| err.to_string())?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if stderr.is_empty() {
        Err("kak -p failed".to_string())
    } else {
        Err(stderr)
    }
}

fn schedule_semantic_refresh(state: &State, session: &str) {
    let token = format!(
        "{}|{}|{}|{}",
        session,
        state.client,
        state.path.display(),
        unique_suffix()
    );
    let _ = fs::write(&state.semantic_file, &token);

    let Ok(exe) = env::current_exe() else {
        return;
    };
    let delay = env::var("KAK_PREVIEW_SYNC_DEBOUNCE").unwrap_or_else(|_| "0.18".to_string());
    let _ = Command::new(exe)
        .arg("--semantic-refresh")
        .arg(&state.semantic_file)
        .arg(&token)
        .arg(session)
        .arg(&state.client)
        .arg(delay)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn run_semantic_refresh(args: &[String]) {
    let (Some(state_file), Some(token), Some(session), Some(client), Some(delay)) = (
        args.get(2),
        args.get(3),
        args.get(4),
        args.get(5),
        args.get(6),
    ) else {
        return;
    };
    let delay = delay.parse::<f64>().unwrap_or(0.18).max(0.0);
    thread::sleep(Duration::from_secs_f64(delay));

    let Ok(current_token) = fs::read_to_string(state_file) else {
        return;
    };
    if current_token.trim() != token {
        return;
    }

    let semantic_cmd = "try %{ lsp-semantic-tokens }";
    let cmd = eval_client_command(client, semantic_cmd);
    let _ = send_to_kak(session, &cmd);
}
