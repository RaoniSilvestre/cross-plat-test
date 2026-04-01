use std::time::Duration;
use tokio::time::sleep;
use tracing::{Span, debug, error, info, instrument, warn};

pub async fn run() {
    process_order(987).await;
}

#[instrument(name = "Checkout", fields(order_id = %id))]
pub async fn process_order(id: u32) {
    info!("Iniciando processamento do pedido");

    if let Err(e) = check_inventory(id).await {
        error!(error = %e, "Falha crítica no checkout");
        return;
    }

    if charge_credit_card(id, 150.00).await.is_err() {
        warn!("Pagamento recusado, tentando novamente...");
    }

    send_notification("sucesso@email.com").await;

    info!("Pedido finalizado com sucesso!");
}

#[instrument(name = "Estoque", skip_all, fields(item_id = %_id))]
async fn check_inventory(_id: u32) -> Result<(), &'static str> {
    debug!("Consultando banco de dados de estoque...");
    sleep(Duration::from_millis(100)).await;

    // Simulando que o estoque está OK
    info!("Item disponível");
    Ok(())
}

#[instrument(name = "Pagamento", skip(amount), fields(valor = %amount))]
async fn charge_credit_card(_id: u32, amount: f64) -> Result<(), ()> {
    debug!("Conectando ao gateway da Stripe...");
    sleep(Duration::from_millis(250)).await;

    Span::current().record("cartao_final", "4242");

    info!("Transação autorizada");
    Ok(())
}

#[instrument(name = "Notificação", skip_all)]
async fn send_notification(email: &str) {
    debug!("Preparando template de e-mail");
    sleep(Duration::from_millis(50)).await;
    info!(target: "email_service", "E-mail enviado para {}", email);
}
