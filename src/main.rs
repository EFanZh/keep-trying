//! Source code for `keep-trying`.

#![warn(
    explicit_outlives_requirements,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_docs,
    noop_method_call,
    pointer_structural_match,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    variant_size_differences,
    clippy::cargo_common_metadata,
    clippy::clone_on_ref_ptr,
    clippy::cognitive_complexity,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::empty_line_after_outer_attr,
    clippy::fallible_impl_from,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::imprecise_flops,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::multiple_inherent_impl,
    clippy::mutex_integer,
    clippy::nonstandard_macro_braces,
    clippy::panic_in_result_fn,
    clippy::path_buf_push_overwrite,
    clippy::pedantic,
    // clippy::print_stderr,
    // clippy::print_stdout,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::suboptimal_flops,
    clippy::suspicious_operation_groupings,
    clippy::todo,
    clippy::trivial_regex,
    clippy::unimplemented,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::use_debug,
    clippy::use_self,
    clippy::useless_let_if_seq,
    clippy::useless_transmute,
    clippy::verbose_file_reads,
    clippy::wildcard_dependencies
)]
#![allow(clippy::non_ascii_literal)]

use std::env;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Command;

fn build_command(program: &OsStr, args: &[OsString]) -> Command {
    let mut command = Command::new(program);

    command.args(args);

    command
}

fn build_command_line_string(program: &OsStr, args: &[OsString]) -> String {
    let mut result = shell_escape::escape(program.to_string_lossy()).into_owned();

    for arg in args {
        result.push(' ');
        result.push_str(&shell_escape::escape(arg.to_string_lossy()));
    }

    result
}

// Just a separator.

fn print_usage(current_exe: &OsStr) {
    let exe_file_name = shell_escape::escape(Path::new(&current_exe).file_name().unwrap().to_string_lossy());

    println!("Usage: {} program arguments …", exe_file_name);
}

fn keep_trying(program: &OsStr, args: &[OsString]) {
    let mut command = build_command(program, args);
    let command_line = build_command_line_string(program, args);

    for i in 1_u64.. {
        match command.status() {
            Ok(status) => {
                if status.success() {
                    // Process exited successfully.

                    break;
                }

                if let Some(code) = status.code() {
                    // Special case for windows.

                    #[cfg(target_os = "windows")]
                    {
                        const STILL_ACTIVE: i32 = 259;

                        if code == STILL_ACTIVE {
                            eprintln!(
                                "Got an exit code of STILL_ACTIVE ({}) which is not a valid exit code, stop trying. \
See https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getexitcodeprocess for \
more information.",
                                STILL_ACTIVE
                            );

                            break;
                        }
                    }

                    // Process exited with a failure exit code, retry.

                    eprintln!("[Exit code is {}, retrying ({} times)] {}", code, i, command_line);
                } else {
                    // Process is killed.

                    break;
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
        None => print_usage(&arguments[0]),
        Some((program, args)) => keep_trying(program, args),
    }
}
