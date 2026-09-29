//! `lab-runner` CLI. Manual argv parsing, following the uor-r4-integer
//! binary pattern. Global flags `--root PATH` / `--ledger-dir PATH` may
//! appear before or after the subcommand; `UOR_RUNNER_ROOT` overrides the
//! default queue root.

use lab_runner::daemon::{self, DaemonConfig};
use lab_runner::{agent, jobs, ledger, RunnerError};
use std::path::PathBuf;
use std::time::Duration;

fn invalid(message: impl Into<String>) -> RunnerError {
    RunnerError::Invalid(message.into())
}

const USAGE: &str = "usage: lab-runner [--root PATH] [--ledger-dir PATH] <command>
  submit <spec.json>        validate and queue a job
  daemon [--poll-ms N] [--monitor-ms N] [--stop-file PATH]
                            run the foreground daemon (launchd KeepAlive owns restarts)
  status [id]               JSON summary of queue/running/done
  tail <id>                 last ~50 lines of the job's logs
  cancel <id>               remove a queued job or kill a running one
  install-agent             write the LaunchAgent plist and print the bootstrap command
  ledger rebuild            recompute model-time.json from the charge/extension records";

struct Globals {
    root: PathBuf,
    ledger_dir: PathBuf,
}

fn parse_globals(args: &[String]) -> Result<(Globals, Vec<String>), RunnerError> {
    let mut root = std::env::var_os("UOR_RUNNER_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(lab_runner::DEFAULT_ROOT));
    let mut ledger_dir: Option<PathBuf> = None;
    let mut rest = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                index += 1;
                root = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| invalid("--root requires a path"))?,
                );
            }
            "--ledger-dir" => {
                index += 1;
                ledger_dir = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| invalid("--ledger-dir requires a path"))?,
                ));
            }
            other => rest.push(other.to_string()),
        }
        index += 1;
    }
    let ledger_dir = match ledger_dir {
        Some(dir) => dir,
        None => lab_runner::default_ledger_dir()?,
    };
    Ok((Globals { root, ledger_dir }, rest))
}

fn flag_value(args: &[String], name: &str) -> Result<Option<String>, RunnerError> {
    let mut index = 0;
    while index < args.len() {
        if args[index] == name {
            return Ok(Some(
                args.get(index + 1)
                    .ok_or_else(|| invalid(format!("{name} requires a value")))?
                    .clone(),
            ));
        }
        index += 1;
    }
    Ok(None)
}

fn run() -> Result<(), RunnerError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (globals, rest) = parse_globals(&args)?;
    match rest.first().map(String::as_str) {
        Some("submit") if rest.len() == 2 => {
            let id = jobs::submit(&globals.root, PathBuf::from(&rest[1]).as_path())?;
            println!("{id}");
            Ok(())
        }
        Some("daemon") => {
            let poll_ms = flag_value(&rest, "--poll-ms")?
                .map(|v| v.parse::<u64>())
                .transpose()
                .map_err(|_| invalid("--poll-ms must be a positive integer"))?
                .unwrap_or(1000);
            let monitor_ms = flag_value(&rest, "--monitor-ms")?
                .map(|v| v.parse::<u64>())
                .transpose()
                .map_err(|_| invalid("--monitor-ms must be a positive integer"))?
                .unwrap_or(2000);
            let stop_file = flag_value(&rest, "--stop-file")?.map(PathBuf::from);
            daemon::run(&DaemonConfig {
                root: globals.root,
                ledger_dir: globals.ledger_dir,
                poll_interval: Duration::from_millis(poll_ms),
                monitor_interval: Duration::from_millis(monitor_ms),
                stop_file,
            })
        }
        Some("status") if rest.len() <= 2 => {
            let summary = jobs::status(&globals.root, rest.get(1).map(String::as_str))?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
            Ok(())
        }
        Some("tail") if rest.len() == 2 => {
            print!("{}", jobs::tail(&globals.root, &rest[1], 50)?);
            Ok(())
        }
        Some("cancel") if rest.len() == 2 => {
            jobs::cancel(&globals.root, &rest[1], &globals.ledger_dir)
        }
        Some("install-agent") if rest.len() == 1 => {
            let program = std::env::current_exe()?;
            let plist = agent::default_plist_path()?;
            let command = agent::write_plist(&plist, &program, &globals.root, &globals.ledger_dir)?;
            println!("wrote {}", plist.display());
            println!("run: {command}");
            Ok(())
        }
        Some("ledger") if rest.len() == 2 && rest[1] == "rebuild" => {
            let state = ledger::rebuild(&globals.ledger_dir)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "cumulative_ms": state.cumulative_ms,
                    "limit_ms": state.limit_ms,
                }))?
            );
            Ok(())
        }
        _ => Err(invalid(USAGE)),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
