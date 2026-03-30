use tracing::{debug, info, instrument};

#[instrument(name = "Serviço")]
pub async fn run() {
    info!("Rodando a lógica assíncrona!");

    inner().await
}

#[instrument(name = "Inner")]
async fn inner() {
    debug!("Inner")
}
