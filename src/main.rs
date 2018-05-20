extern crate shell_escape;

use std::env;
use std::ffi::{OsStr, OsString};
use std::iter;
use std::path::Path;
use std::process::Command;

fn create_command(program: &OsStr, args: &[OsString]) -> Command {
    let mut command = Command::new(program);

    command.args(args);

    command
}

fn create_command_line(program: &OsStr, args: &[OsString]) -> String {
    iter::once(program)
        .chain(args.into_iter().map(|x| x.as_ref()))
        .map(|x| shell_escape::escape(x.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ")
}

// Just a separator.

fn usage(current_exe: &OsStr) {
    let exe_file_name = Path::new(&current_exe).file_name().unwrap().to_string_lossy();

    println!("Usage: {} program arguments …", exe_file_name);
}

fn keep_trying(program: &OsStr, args: &[OsString]) {
    let mut command = create_command(program, args);
    let command_line = create_command_line(program, args);

    for i in 1u64.. {
        match command.status() {
            Ok(status) => {
                if status.success() {
                    // Process exited successfully.
                    break;
                } else {
                    match status.code() {
                        None => {
                            // Process is killed.
                            break;
                        }
                        Some(code) => {
                            // Special case for windows.
                            if cfg!(target_os = "windows") {
                                const STILL_ACTIVE: u32 = 259;

                                if code == STILL_ACTIVE as _ {
                                    eprintln!(
"Got an exit code of STILL_ACTIVE ({}) which is not a valid exit code, stop trying. \
See https://msdn.microsoft.com/en-us/library/windows/desktop/ms683189.aspx for more information",
                                        STILL_ACTIVE
                                    );
                                    break;
                                }
                            }

                            // Process exited with a failure exit code, retry.
                            eprintln!("[Exit code is {}, retrying ({} times)] {}", code, i, command_line);
                        }
                    }
                }
            }
            Err(error) => {
                // Failed to start process.
                eprintln!("Failed to run “{}”: {}", command_line, error);
                break;
            }
        }
    }
}

fn main() {
    let arguments = env::args_os().collect::<Vec<_>>();

    match arguments[1..].split_first() {
        None => usage(&arguments[0]),
        Some((program, args)) => keep_trying(program, args),
    }
}
