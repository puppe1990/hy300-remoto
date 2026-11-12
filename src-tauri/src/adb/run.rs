//! Spawn `adb` with piped stdio. Raw screencap is ~2.6s; kill at `SCREENCAP_TIMEOUT`.

use super::AdbError;
use std::io::Read;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub(super) fn run_limited(
    bin: &str,
    args: &[String],
    timeout: Option<Duration>,
) -> Result<Vec<u8>, AdbError> {
    let mut child = spawn_piped(bin, args)?;
    let stdout_h = spawn_reader(take_pipe(child.stdout.take(), "stdout")?);
    let stderr_h = spawn_reader(take_pipe(child.stderr.take(), "stderr")?);
    let status = wait_child(&mut child, bin, args, timeout)?;
    let stdout = join_reader(stdout_h, "stdout")?;
    let stderr = join_reader(stderr_h, "stderr")?;
    reject_failure(bin, args, status, &stderr)?;
    Ok(stdout)
}

fn spawn_piped(bin: &str, args: &[String]) -> Result<Child, AdbError> {
    Command::new(bin)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AdbError::new(format!("não executou {bin} {}: {e}", args.join(" "))))
}

fn take_pipe<T>(pipe: Option<T>, name: &str) -> Result<T, AdbError> {
    pipe.ok_or_else(|| AdbError::new(format!("adb sem {name}; expected piped {name}")))
}

fn spawn_reader<R: Read + Send + 'static>(mut pipe: R) -> JoinHandle<std::io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut buf = Vec::new();
        pipe.read_to_end(&mut buf).map(|_| buf)
    })
}

fn wait_child(
    child: &mut Child,
    bin: &str,
    args: &[String],
    timeout: Option<Duration>,
) -> Result<ExitStatus, AdbError> {
    match timeout {
        None => child
            .wait()
            .map_err(|e| AdbError::new(format!("wait {}: {e}", args.join(" ")))),
        Some(limit) => wait_deadline(child, bin, args, limit),
    }
}

fn wait_deadline(
    child: &mut Child,
    bin: &str,
    args: &[String],
    limit: Duration,
) -> Result<ExitStatus, AdbError> {
    let start = Instant::now();
    loop {
        if let Some(st) = poll_child(child, args)? {
            return Ok(st);
        }
        if start.elapsed() >= limit {
            return kill_timed_out(child, bin, args, limit);
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn poll_child(child: &mut Child, args: &[String]) -> Result<Option<ExitStatus>, AdbError> {
    child
        .try_wait()
        .map_err(|e| AdbError::new(format!("wait {}: {e}", args.join(" "))))
}

fn kill_timed_out(
    child: &mut Child,
    bin: &str,
    args: &[String],
    limit: Duration,
) -> Result<ExitStatus, AdbError> {
    let _ = child.kill();
    let _ = child.wait();
    Err(AdbError::new(format!(
        "{bin} {} estourou {}s",
        args.join(" "),
        limit.as_secs()
    )))
}

fn join_reader(
    handle: JoinHandle<std::io::Result<Vec<u8>>>,
    name: &str,
) -> Result<Vec<u8>, AdbError> {
    handle
        .join()
        .map_err(|_| {
            AdbError::new(format!(
                "{name} join panicked; expected io::Result<Vec<u8>>"
            ))
        })?
        .map_err(|e| {
            AdbError::new(format!(
                "{name} read failed: {e}; expected piped stdout/stderr bytes"
            ))
        })
}

fn reject_failure(
    bin: &str,
    args: &[String],
    status: ExitStatus,
    stderr: &[u8],
) -> Result<(), AdbError> {
    if status.success() {
        return Ok(());
    }
    Err(AdbError::new(format!(
        "{bin} {} saiu {status}: {}",
        args.join(" "),
        String::from_utf8_lossy(stderr).trim()
    )))
}
