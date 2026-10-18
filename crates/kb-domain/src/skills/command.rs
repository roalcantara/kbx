//! Runs the Skills command: its status, stdout and stderr in full, or nothing when cancelled.
use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// How often a running command checks for a cancellation.
const POLL: Duration = Duration::from_millis(20);

/// What a finished command printed.
#[derive(Debug)]
pub struct CommandOutput {
    /// How it exited.
    pub status: ExitStatus,
    /// Everything it wrote to stdout.
    pub stdout: String,
    /// Everything it wrote to stderr.
    pub stderr: String,
}

/// Why a command gave no output.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    /// It could not be started.
    #[error("cannot run {program}: {source}")]
    Spawn {
        /// The command.
        program: String,
        /// The OS error.
        source: io::Error,
    },
    /// Its output could not be read, or was not UTF-8.
    #[error("cannot read the output of {program}: {source}")]
    Output {
        /// The command.
        program: String,
        /// The I/O error.
        source: io::Error,
    },
    /// `cancel` was set; the command was killed.
    #[error("cancelled")]
    Cancelled,
}

/// Runs `program` with `args`, no stdin, until it exits or `cancel` is set (then it is killed).
///
/// # Errors
///
/// It cannot start, its output cannot be read, or it was cancelled.
pub fn run_command(program: &Path, args: &[&str], cancel: &AtomicBool) -> Result<CommandOutput, CommandError> {
    let name = program.display().to_string();
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| CommandError::Spawn {
            program: name.clone(),
            source,
        })?;
    // Each pipe drains on its own thread, so a command that fills one never blocks on it.
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let status = loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(CommandError::Cancelled);
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(POLL),
            Err(source) => return Err(CommandError::Output { program: name, source }),
        }
    };
    let collect = |handle: JoinHandle<io::Result<String>>| {
        handle
            .join()
            .unwrap_or_else(|_| Err(io::Error::other("the output reader stopped")))
            .map_err(|source| CommandError::Output {
                program: name.clone(),
                source,
            })
    };
    Ok(CommandOutput {
        status,
        stdout: collect(stdout)?,
        stderr: collect(stderr)?,
    })
}

/// Reads `pipe` to the end on a thread of its own.
fn drain(pipe: Option<impl Read + Send + 'static>) -> JoinHandle<io::Result<String>> {
    thread::spawn(move || {
        let mut text = String::new();
        if let Some(mut pipe) = pipe {
            pipe.read_to_string(&mut text)?;
        }
        Ok(text)
    })
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn returns_the_status_and_both_streams() {
        let output = run_command(
            Path::new("/bin/sh"),
            &["-c", "echo '[]'; echo warn >&2; exit 3"],
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(
            (output.stdout.as_str(), output.stderr.as_str(), output.status.code()),
            ("[]\n", "warn\n", Some(3))
        );
    }

    #[test]
    fn kills_a_cancelled_command() {
        let started = Instant::now();
        let result = run_command(Path::new("/bin/sleep"), &["5"], &AtomicBool::new(true));
        assert!(matches!(result, Err(CommandError::Cancelled)));
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn says_why_a_command_cannot_run() {
        let error = run_command(Path::new("/nonexistent/skills"), &[], &AtomicBool::new(false)).unwrap_err();
        assert!(
            error.to_string().starts_with("cannot run /nonexistent/skills: "),
            "{error}"
        );
    }
}
