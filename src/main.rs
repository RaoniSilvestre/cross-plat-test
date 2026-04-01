use clap::Parser;

#[cfg(target_os = "linux")]
use crate::setup::linux;

#[cfg(target_os = "windows")]
use crate::setup::windows;

mod cli;
mod log;
mod service;
mod setup;

fn main() {
    let args = cli::CLI::parse();

    // Não mexer, essa variável precisa se manter viva até o final do programa para logar
    // corretamente.
    let _log_handle = log::init_logger(args.log_path.into());

    #[cfg(target_os = "linux")]
    linux::linux_setup();

    #[cfg(target_os = "windows")]
    windows::windows_setup();
}
