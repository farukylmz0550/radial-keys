use super::presets::{self, BuiltinMacroConfig, CommandSpec, MacroInvocation};
use nix::{
    sys::signal::{Signal, kill},
    unistd::Pid,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::{
    collections::{BTreeMap, VecDeque},
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
    sync::{Mutex, oneshot},
};
use uuid::Uuid;

const OUTPUT_LIMIT: usize = 16 * 1024;
const RESULT_LIMIT: usize = 20;

#[derive(Debug, Clone, Serialize)]
pub struct RunningMacroInfo {
    pub id: String,
    pub slot: usize,
    pub label: String,
    pub started_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroResult {
    pub id: String,
    pub slot: usize,
    pub label: String,
    pub finished_unix_ms: u128,
    pub outcome: String,
    pub exit_code: Option<i32>,
    pub error_output: String,
}

#[derive(Debug, Error)]
pub enum MacroError {
    #[error("Python macro path must be absolute and end with .py")]
    InvalidPath,
    #[error("Python macro file does not exist or is not a regular file")]
    MissingScript,
    #[error("Python macro changed since it was accepted; review it and accept the new version")]
    ScriptChanged,
    #[error("built-in macro is invalid: {0}")]
    InvalidPreset(String),
    #[error("this macro is already running")]
    AlreadyRunning,
    #[error("could not start macro: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("KDE desktop toggle is unavailable: {0}")]
    Desktop(String),
}

struct RunningEntry {
    info: RunningMacroInfo,
    identity: String,
    stop: Option<oneshot::Sender<StopReason>>,
}

#[derive(Debug, Clone, Copy)]
enum StopReason {
    Cancelled,
}

#[derive(Default)]
struct ProcessState {
    running: BTreeMap<String, RunningEntry>,
    results: VecDeque<MacroResult>,
}

#[derive(Clone, Default)]
pub struct MacroProcessManager {
    state: Arc<Mutex<ProcessState>>,
}

impl MacroProcessManager {
    pub async fn start_python(
        &self,
        slot: usize,
        label: String,
        script: &str,
        expected_hash: Option<&str>,
        single_instance: bool,
        timeout_seconds: Option<u64>,
    ) -> Result<String, MacroError> {
        let path = validate_script_path(script)?;
        let actual_hash = sha256_file(&path).map_err(|_| MacroError::MissingScript)?;
        if expected_hash.is_some_and(|expected| expected != actual_hash) {
            return Err(MacroError::ScriptChanged);
        }
        let identity = format!("python:{}", path.display());
        self.start_command(
            slot,
            label,
            identity,
            CommandSpec {
                program: "python3".into(),
                args: vec![path.display().to_string()],
            },
            single_instance,
            timeout_seconds,
        )
        .await
    }

    pub async fn start_preset(
        &self,
        slot: usize,
        label: String,
        preset: &BuiltinMacroConfig,
        single_instance: bool,
        timeout_seconds: Option<u64>,
    ) -> Result<String, MacroError> {
        let invocation = presets::invocation(preset).map_err(MacroError::InvalidPreset)?;
        let command = match invocation {
            MacroInvocation::Process(command) => command,
            MacroInvocation::ToggleDesktop => toggle_desktop_command().await?,
        };
        // Desktop tools such as playerctl, wpctl, Spectacle and user-selected
        // applications belong to the host session, not the KDE runtime. Run
        // built-in actions through Flatpak's explicit host launcher. Python
        // macros continue through start_python and remain inside the sandbox.
        let mut host_args = vec!["--host".to_owned(), command.program];
        host_args.extend(command.args);
        let host_command = CommandSpec {
            program: "flatpak-spawn".into(),
            args: host_args,
        };
        self.start_command(
            slot,
            label,
            format!("preset:{}", preset.id),
            host_command,
            single_instance,
            timeout_seconds,
        )
        .await
    }

    async fn start_command(
        &self,
        slot: usize,
        label: String,
        identity: String,
        specification: CommandSpec,
        single_instance: bool,
        timeout_seconds: Option<u64>,
    ) -> Result<String, MacroError> {
        let mut state = self.state.lock().await;
        if single_instance
            && state
                .running
                .values()
                .any(|running| running.identity == identity)
        {
            return Err(MacroError::AlreadyRunning);
        }

        let mut command = Command::new(&specification.program);
        command
            .args(&specification.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.as_std_mut().process_group(0);
        }
        let mut child = command.spawn().map_err(MacroError::Spawn)?;
        let pid = child.id().map(|pid| pid as i32);
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let id = Uuid::new_v4().to_string();
        let started_unix_ms = unix_time_ms();
        let info = RunningMacroInfo {
            id: id.clone(),
            slot,
            label: label.clone(),
            started_unix_ms,
        };
        let (stop, stop_receiver) = oneshot::channel();
        state.running.insert(
            id.clone(),
            RunningEntry {
                info,
                identity,
                stop: Some(stop),
            },
        );
        drop(state);

        let shared_state = self.state.clone();
        let worker_id = id.clone();
        tokio::spawn(async move {
            let stdout_reader = stdout.map(|stream| tokio::spawn(read_capped(stream)));
            let stderr_reader = stderr.map(|stream| tokio::spawn(read_capped(stream)));
            let mut group_guard = ProcessGroupGuard { pid, active: true };
            let outcome = monitor_child(&mut child, pid, stop_receiver, timeout_seconds).await;
            if matches!(
                outcome,
                Outcome::Exited(_) | Outcome::Cancelled | Outcome::TimedOut
            ) {
                group_guard.active = false;
            }
            let _ = child.wait().await;
            let stdout_text = read_result(stdout_reader).await;
            let stderr_text = read_result(stderr_reader).await;
            let error_output = if !stderr_text.is_empty() {
                stderr_text
            } else {
                stdout_text
            };
            let result = MacroResult {
                id: worker_id.clone(),
                slot,
                label,
                finished_unix_ms: unix_time_ms(),
                outcome: outcome.name().into(),
                exit_code: outcome.exit_code(),
                error_output,
            };
            match outcome {
                Outcome::Exited(Some(0)) => tracing::info!(slot = slot + 1, "macro completed"),
                Outcome::Exited(code) => {
                    tracing::warn!(slot = slot + 1, ?code, "macro exited with an error")
                }
                Outcome::TimedOut => {
                    tracing::warn!(slot = slot + 1, "macro timed out and was terminated")
                }
                Outcome::Cancelled => {
                    tracing::info!(slot = slot + 1, "macro was stopped by the user")
                }
                Outcome::WaitFailed => {
                    tracing::warn!(slot = slot + 1, "could not collect macro exit status")
                }
            }
            let mut state = shared_state.lock().await;
            state.running.remove(&worker_id);
            state.results.push_front(result);
            state.results.truncate(RESULT_LIMIT);
        });
        tracing::info!(slot = slot + 1, "started macro process");
        Ok(id)
    }

    pub async fn terminate(&self, id: &str) -> bool {
        let mut state = self.state.lock().await;
        state
            .running
            .get_mut(id)
            .and_then(|running| running.stop.take())
            .is_some_and(|stop| stop.send(StopReason::Cancelled).is_ok())
    }

    pub async fn terminate_slot(&self, slot: usize) -> usize {
        let mut state = self.state.lock().await;
        let mut stopped = 0;
        for running in state.running.values_mut() {
            if running.info.slot == slot
                && running
                    .stop
                    .take()
                    .is_some_and(|stop| stop.send(StopReason::Cancelled).is_ok())
            {
                stopped += 1;
            }
        }
        stopped
    }

    pub async fn terminate_all(&self) {
        let mut state = self.state.lock().await;
        for running in state.running.values_mut() {
            if let Some(stop) = running.stop.take() {
                let _ = stop.send(StopReason::Cancelled);
            }
        }
        drop(state);
        for _ in 0..35 {
            if self.state.lock().await.running.is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    pub async fn snapshot(&self) -> (Vec<RunningMacroInfo>, Vec<MacroResult>) {
        let state = self.state.lock().await;
        (
            state
                .running
                .values()
                .map(|entry| entry.info.clone())
                .collect(),
            state.results.iter().cloned().collect(),
        )
    }
}

pub fn validate_script_path(script: &str) -> Result<PathBuf, MacroError> {
    let path = Path::new(script);
    if !path.is_absolute() || path.extension().is_none_or(|extension| extension != "py") {
        return Err(MacroError::InvalidPath);
    }
    let canonical = path.canonicalize().map_err(|_| MacroError::MissingScript)?;
    if canonical
        .extension()
        .is_none_or(|extension| extension != "py")
    {
        return Err(MacroError::InvalidPath);
    }
    let metadata = std::fs::metadata(&canonical).map_err(|_| MacroError::MissingScript)?;
    if !metadata.is_file() {
        return Err(MacroError::MissingScript);
    }
    Ok(canonical)
}

pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

async fn toggle_desktop_command() -> Result<CommandSpec, MacroError> {
    let query = tokio::time::timeout(
        Duration::from_secs(5),
        Command::new("flatpak-spawn")
            .args([
                "--host",
                "qdbus6",
                "org.kde.KWin",
                "/KWin",
                "org.kde.KWin.showingDesktop",
            ])
            .output(),
    )
    .await
    .map_err(|_| MacroError::Desktop("KWin query timed out".into()))?
    .map_err(|error| MacroError::Desktop(error.to_string()))?;
    if !query.status.success() {
        return Err(MacroError::Desktop(
            String::from_utf8_lossy(&query.stderr).trim().into(),
        ));
    }
    let currently_shown = String::from_utf8_lossy(&query.stdout)
        .trim()
        .eq_ignore_ascii_case("true");
    Ok(CommandSpec {
        program: "qdbus6".into(),
        args: vec![
            "org.kde.KWin".into(),
            "/KWin".into(),
            "org.kde.KWin.showDesktop".into(),
            (!currently_shown).to_string(),
        ],
    })
}

#[derive(Debug)]
enum Outcome {
    Exited(Option<i32>),
    Cancelled,
    TimedOut,
    WaitFailed,
}

impl Outcome {
    fn name(&self) -> &'static str {
        match self {
            Self::Exited(Some(0)) => "completed",
            Self::Exited(_) => "failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::WaitFailed => "wait_failed",
        }
    }

    fn exit_code(&self) -> Option<i32> {
        match self {
            Self::Exited(code) => *code,
            _ => None,
        }
    }
}

async fn monitor_child(
    child: &mut Child,
    pid: Option<i32>,
    mut stop: oneshot::Receiver<StopReason>,
    timeout_seconds: Option<u64>,
) -> Outcome {
    if let Some(seconds) = timeout_seconds {
        tokio::select! {
            status = child.wait() => status.map(|status| Outcome::Exited(status.code())).unwrap_or(Outcome::WaitFailed),
            reason = &mut stop => {
                let _reason = reason.unwrap_or(StopReason::Cancelled);
                terminate_process_group(pid, child).await;
                Outcome::Cancelled
            }
            _ = tokio::time::sleep(Duration::from_secs(seconds)) => {
                terminate_process_group(pid, child).await;
                Outcome::TimedOut
            }
        }
    } else {
        tokio::select! {
            status = child.wait() => status.map(|status| Outcome::Exited(status.code())).unwrap_or(Outcome::WaitFailed),
            reason = &mut stop => {
                let _reason = reason.unwrap_or(StopReason::Cancelled);
                terminate_process_group(pid, child).await;
                Outcome::Cancelled
            }
        }
    }
}

async fn terminate_process_group(pid: Option<i32>, child: &mut Child) {
    #[cfg(unix)]
    if let Some(pid) = pid {
        let _ = kill(Pid::from_raw(-pid), Signal::SIGTERM);
    }
    let waited = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
    if waited.is_err() {
        let _ = child.start_kill();
        let _ = child.wait().await;
    }
    #[cfg(unix)]
    if let Some(pid) = pid {
        for _ in 0..20 {
            if kill(Pid::from_raw(-pid), None).is_err() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let _ = kill(Pid::from_raw(-pid), Signal::SIGKILL);
    }
}

struct ProcessGroupGuard {
    pid: Option<i32>,
    active: bool,
}

impl Drop for ProcessGroupGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        if self.active
            && let Some(pid) = self.pid
        {
            let _ = kill(Pid::from_raw(-pid), Signal::SIGKILL);
        }
    }
}

async fn read_capped<R: AsyncRead + Unpin>(mut stream: R) -> String {
    let mut output = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) | Err(_) => break,
            Ok(count) => {
                let remaining = OUTPUT_LIMIT.saturating_sub(output.len());
                output.extend_from_slice(&buffer[..count.min(remaining)]);
            }
        }
    }
    String::from_utf8_lossy(&output).trim().to_owned()
}

async fn read_result(reader: Option<tokio::task::JoinHandle<String>>) -> String {
    match reader {
        Some(mut reader) => match tokio::time::timeout(Duration::from_secs(1), &mut reader).await {
            Ok(Ok(output)) => output,
            _ => {
                reader.abort();
                String::new()
            }
        },
        None => String::new(),
    }
}

fn unix_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt};

    #[test]
    fn validates_absolute_python_paths_and_resolves_symlinks() {
        let directory = tempfile::tempdir().unwrap();
        let script = directory.path().join("macro.py");
        fs::write(&script, "print('ok')").unwrap();
        let link = directory.path().join("alias.py");
        std::os::unix::fs::symlink(&script, &link).unwrap();
        assert_eq!(
            validate_script_path(link.to_str().unwrap()).unwrap(),
            script
        );
        assert!(matches!(
            validate_script_path("relative.py"),
            Err(MacroError::InvalidPath)
        ));
        assert!(matches!(
            validate_script_path("/tmp/missing.py"),
            Err(MacroError::MissingScript)
        ));
    }

    #[test]
    fn calculates_sha256_and_detects_content_changes() {
        let directory = tempfile::tempdir().unwrap();
        let script = directory.path().join("macro.py");
        fs::write(&script, "print('one')").unwrap();
        let original = sha256_file(&script).unwrap();
        fs::write(&script, "print('two')").unwrap();
        assert_ne!(original, sha256_file(&script).unwrap());
        assert_eq!(original.len(), 64);
    }

    #[tokio::test]
    async fn single_instance_prevents_duplicate_execution_and_allows_restart() {
        let manager = MacroProcessManager::default();
        let directory = tempfile::tempdir().unwrap();
        let script = directory.path().join("wait.py");
        fs::write(&script, "import time; time.sleep(.25)").unwrap();
        let hash = sha256_file(&script).unwrap();
        manager
            .start_python(
                0,
                "wait".into(),
                script.to_str().unwrap(),
                Some(&hash),
                true,
                None,
            )
            .await
            .unwrap();
        assert!(matches!(
            manager
                .start_python(
                    0,
                    "wait".into(),
                    script.to_str().unwrap(),
                    Some(&hash),
                    true,
                    None
                )
                .await,
            Err(MacroError::AlreadyRunning)
        ));
        tokio::time::sleep(Duration::from_millis(400)).await;
        assert!(manager.snapshot().await.0.is_empty());
        manager
            .start_python(
                0,
                "wait".into(),
                script.to_str().unwrap(),
                Some(&hash),
                true,
                None,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn records_python_failure_and_captures_diagnostics() {
        let manager = MacroProcessManager::default();
        let directory = tempfile::tempdir().unwrap();
        let script = directory.path().join("failure.py");
        fs::write(
            &script,
            "import sys; print('macro diagnostic', file=sys.stderr); raise SystemExit(7)",
        )
        .unwrap();
        let hash = sha256_file(&script).unwrap();
        let id = manager
            .start_python(
                2,
                "failure".into(),
                script.to_str().unwrap(),
                Some(&hash),
                true,
                None,
            )
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                let (running, results) = manager.snapshot().await;
                if running.is_empty() {
                    let result = results.iter().find(|result| result.id == id).unwrap();
                    assert_eq!(result.outcome, "failed");
                    assert_eq!(result.exit_code, Some(7));
                    assert!(result.error_output.contains("macro diagnostic"));
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("failed macro should be reaped and reported");
    }

    #[tokio::test]
    async fn disabled_single_instance_allows_concurrent_runs() {
        let manager = MacroProcessManager::default();
        let directory = tempfile::tempdir().unwrap();
        let script = directory.path().join("concurrent.py");
        fs::write(&script, "import time; time.sleep(10)").unwrap();
        let hash = sha256_file(&script).unwrap();
        for _ in 0..2 {
            manager
                .start_python(
                    1,
                    "parallel".into(),
                    script.to_str().unwrap(),
                    Some(&hash),
                    false,
                    None,
                )
                .await
                .unwrap();
        }
        let (running, _) = manager.snapshot().await;
        assert_eq!(running.len(), 2);
        assert_eq!(manager.terminate_slot(1).await, 2);
        tokio::time::timeout(Duration::from_secs(4), async {
            while !manager.snapshot().await.0.is_empty() {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        })
        .await
        .expect("both cancelled scripts should be reaped");
    }

    #[tokio::test]
    async fn timeout_and_termination_reap_child_processes() {
        let manager = MacroProcessManager::default();
        let directory = tempfile::tempdir().unwrap();
        let script = directory.path().join("wait.py");
        fs::write(&script, "import time; time.sleep(20)").unwrap();
        let hash = sha256_file(&script).unwrap();
        let id = manager
            .start_python(
                0,
                "wait".into(),
                script.to_str().unwrap(),
                Some(&hash),
                true,
                Some(1),
            )
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(1200)).await;
        assert!(manager.snapshot().await.0.is_empty());
        assert!(
            manager
                .snapshot()
                .await
                .1
                .iter()
                .any(|result| result.id == id && result.outcome == "timed_out")
        );
        let id = manager
            .start_python(
                0,
                "wait".into(),
                script.to_str().unwrap(),
                Some(&hash),
                true,
                None,
            )
            .await
            .unwrap();
        assert!(manager.terminate(&id).await);
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(manager.snapshot().await.0.is_empty());
    }

    #[test]
    fn config_files_are_written_with_private_permissions() {
        let directory = tempfile::tempdir().unwrap();
        let store =
            crate::backend::config::ConfigStore::new(directory.path().join("private/config.toml"));
        store
            .save(&crate::backend::config::RadialConfig::default())
            .unwrap();
        let metadata = fs::metadata(directory.path().join("private/config.toml")).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
        let parent = fs::metadata(directory.path().join("private")).unwrap();
        assert_eq!(parent.permissions().mode() & 0o777, 0o700);
    }
}
