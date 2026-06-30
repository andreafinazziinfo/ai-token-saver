use anyhow::Result;
use clap::Parser;

mod agents;
mod artifact;
mod benchmark;
mod cli;
mod dashboard;
mod dispatch;
mod distiller;
mod doctor;
mod dotnet;
mod filter_pipeline;
mod index_cli;
mod plugins;
mod rewrite;
mod setup;
mod sync_rules;
mod validate;

#[cfg(test)]
mod fuzz_tests;

fn main() {
    reset_sigpipe();
    let result: Result<()> = dispatch::dispatch(cli::Cli::parse().command);
    if let Err(e) = result {
        eprintln!("rtk: {e}");
        std::process::exit(1);
    }
}

/// Restore the default SIGPIPE disposition so piping RTK output into `head`,
/// `less`, etc. terminates the process quietly on a closed pipe instead of
/// panicking on EPIPE (which surfaced as exit code 101).
#[cfg(unix)]
fn reset_sigpipe() {
    // SAFETY: resetting a signal to its default handler is async-signal-safe
    // and is the standard pattern for well-behaved Unix CLIs.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn reset_sigpipe() {}
