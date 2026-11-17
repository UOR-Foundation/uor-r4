#![forbid(unsafe_code)]

use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

use uor_r4_mvm::MvmSandbox;

const USAGE: &str = "Usage: uor-mvm status | uor-mvm collatz <1..512>";
const MAX_START: u16 = 512;
const COLLATZ: &str = include_str!("../workloads/collatz.c");

fn run(args: &[OsString]) -> Result<(), String> {
    let limit = match args {
        [command] if command == "status" => None,
        [command, value] if command == "collatz" => {
            let limit = value
                .to_str()
                .and_then(|text| text.parse::<u16>().ok())
                .filter(|limit| (1..=MAX_START).contains(limit))
                .ok_or_else(|| format!("Collatz limit must be an integer from 1 to {MAX_START}"))?;
            Some(limit)
        }
        _ => return Err(USAGE.into()),
    };

    let mut vm = MvmSandbox::boot_default().map_err(|error| error.to_string())?;
    if let Some(limit) = limit {
        let source = format!("long limit = {limit};\n{COLLATZ}");
        vm.write_guest_file("/workspace/collatz.c", source.as_bytes())
            .map_err(|error| error.to_string())?;
        let outcome = vm
            .exec_command("cc collatz.c -o collatz && ./collatz", "/workspace")
            .map_err(|error| error.to_string())?;
        if outcome.exit_code != 0 {
            return Err(format!(
                "guest exit {}: {}",
                outcome.exit_code, outcome.stderr
            ));
        }
        if outcome.compiled_elf_machine != Some(243) || outcome.instructions_retired == 0 {
            return Err("expected the guest to compile and execute a RISC-V ELF".into());
        }
        print!("{}", outcome.stdout);
    } else {
        println!(
            "architecture={} guest_cid={}",
            vm.architecture(),
            vm.guest_cid()
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(&env::args_os().skip(1).collect::<Vec<_>>()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
