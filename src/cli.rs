use clap::Parser;

/// Um coletor?? quem sabe??
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CLI {
    /// Caminho para a pasta de logs
    #[arg(short, long, default_value = "logs")]
    pub log_path: String,
}
