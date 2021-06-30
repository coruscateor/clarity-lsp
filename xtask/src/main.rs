//! See https://github.com/matklad/cargo-xtask/.
//!
//! This binary defines various auxiliary build commands, which are not
//! expressible with just `cargo`. Notably, it provides `cargo xtask codegen`
//! for code generation and `cargo xtask install` for installation of
//! clarity-lsp server and client.
//!
//! This binary is integrated into the `cargo` command line by using an alias in
//! `.cargo/config`.

use std::env;

use pico_args::Arguments;
use xtask::{
    dist::{run_dist, ClientOpts},
    install::{ClientOpt, InstallCmd, ServerOpt},
    not_bash::pushd,
    pre_commit, project_root, run_pre_cache, run_release, Result,
};

fn main() -> Result<()> {
    if env::args().next().map(|it| it.contains("pre-commit")) == Some(true) {
        return pre_commit::run_hook();
    }

    let _d = pushd(project_root());

    let mut args = Arguments::from_env();
    let subcommand = args.subcommand()?.unwrap_or_default();

    match subcommand.as_str() {
        "install" => {
            if args.contains(["-h", "--help"]) {
                eprintln!(
                    "\
cargo xtask install
Install clarity-lsp server or editor plugin.

USAGE:
    cargo xtask install [FLAGS]

FLAGS:
        --server         Install only the language server
        --jemalloc       Use jemalloc for server
    -h, --help           Prints help information
        "
                );
                return Ok(());
            }
            let server = args.contains("--server");
            let client_code = args.contains("--client-code");
            if server && client_code {
                eprintln!(
                    "error: The argument `--server` cannot be used with `--client-code`\n\n\
                     For more information try --help"
                );
                return Ok(());
            }

            let jemalloc = args.contains("--jemalloc");

            args.finish()?;

            InstallCmd {
                client: if server {
                    None
                } else {
                    Some(ClientOpt::VsCode)
                },
                server: if client_code {
                    None
                } else {
                    Some(ServerOpt { jemalloc })
                },
            }
            .run()
        }
        "install-pre-commit-hook" => {
            args.finish()?;
            pre_commit::install_hook()
        }
        "pre-cache" => {
            args.finish()?;
            run_pre_cache()
        }
        "release" => {
            let dry_run = args.contains("--dry-run");
            args.finish()?;
            run_release(dry_run)
        }
        "dist" => {
            let client_opts = if args.contains("--client") {
                Some(ClientOpts {
                    version: args.value_from_str("--version")?,
                    release_tag: args.value_from_str("--tag")?,
                })
            } else {
                None
            };
            args.finish()?;
            run_dist(client_opts)
        }
        _ => {
            eprintln!(
                "\
cargo xtask
Run custom build command.

USAGE:
    cargo xtask <SUBCOMMAND>

SUBCOMMANDS:
    format
    install-pre-commit-hook
    install
    dist"
            );
            Ok(())
        }
    }
}
