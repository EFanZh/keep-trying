extern crate shell_escape;

use std::env;
use std::ffi::{OsStr, OsString};
use std::iter;
use std::path::Path;
use std::process::Command;

fn create_command_line(program: &OsStr, args: &[OsString]) -> String {
    iter::once(program)
        .chain(args.into_iter().map(|x| x.as_ref()))
        .map(|x| shell_escape::escape(x.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn create_command(program: &OsStr, args: &[OsString]) -> Command {
    let mut command = Command::new(program);

    command.args(args);

    command
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
                                const STILL_ALIVE: u32 = 259;

                                if code == STILL_ALIVE as _ {
                                    eprintln!(
"Got an exit code of STILL_ALIVE ({}) which is not a valid exit code, stop trying. \
See https://msdn.microsoft.com/en-us/library/windows/desktop/ms683189.aspx for more information",
                                        STILL_ALIVE
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
                eprintln!("Fail to run “{}”: {}", command_line, error);
                break;
            }
        }
    }
}

fn main() {
    let mut arguments = env::args_os();

    if arguments.len() < 2 {
        usage(&arguments.next().unwrap());
    } else {
        let program = arguments.nth(1).unwrap();
        let args = arguments.collect::<Vec<_>>();

        keep_trying(&program, &args);
    }
}
