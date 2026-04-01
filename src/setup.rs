#[cfg(target_os = "linux")]
pub mod linux {
    use tracing::instrument;
    pub use tracing::{error, info, warn};

    #[instrument(name = "Linux")]
    pub fn linux_setup() {
        use sd_notify::{NotifyState, notify};
        use tokio::signal::unix::{SignalKind, signal};

        let rt = tokio::runtime::Builder::new_multi_thread()
            .build()
            .expect("Falha ao criar Runtime multithreaded");

        rt.block_on(async {
            info!("Inicializando serviço");
            let service_handle = tokio::spawn(crate::service::run());

            if let Err(e) = notify(&[NotifyState::Ready]) {
                error!("Aviso: Falha ao notificar systemd: {}", e);
            }

            let Ok(mut sigterm) = signal(SignalKind::terminate()) else {
                error!("Erro ao recuperar SIGTERM");
                return;
            };

            let Ok(mut sigint) = signal(SignalKind::interrupt()) else {
                error!("Erro ao recuperar SIGINT");
                return;
            };

            tokio::select! {
                _ = service_handle => {
                    warn!("A lógica do serviço terminou sozinha.");
                }
                _ = sigterm.recv() => {
                    warn!("SIGTERM recebido. Encerrando graciosamente...");
                }
                _ = sigint.recv() => {
                    warn!("SIGINT (Ctrl+C) recebido. Encerrando...");
                }
            }

            info!("Serviço finalizado.");
        });
    }
}

/// Sei pouco sobre essa parte aqui, tirei muita coisa da documentação da lib `windows_service` e
/// confiei no processo, mas não executei no windows pra ver se funciona.
///
/// O teste feito até então foi uma cross-compilation setando a arquitetura target no `cargo build`
/// buildou direitinho kkkk
#[cfg(target_os = "windows")]
pub mod windows {
    use std::ffi::OsString;
    use std::time::Duration;
    use tokio_util::sync::CancellationToken;
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
    };

    use tracing::{info, warn};

    define_windows_service!(ffi_service_main, system_service_main);

    pub fn windows_setup() {
        if let Err(e) = service_dispatcher::start("MeuServico", ffi_service_main) {
            warn!("Erro ao iniciar dispatcher: {:?}", e);
        }
    }

    pub fn system_service_main(_arguments: Vec<OsString>) {
        let token = CancellationToken::new();
        let token_for_handler = token.clone();

        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Stop | ServiceControl::Shutdown => {
                    token_for_handler.cancel();
                    ServiceControlHandlerResult::NoError
                }
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        let status_handle = service_control_handler::register("MeuServico", event_handler).unwrap();

        status_handle
            .set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })
            .unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            info!("Serviço Windows iniciado!");

            tokio::select! {
                _ = crate::service::run() => {},
                _ = token.cancelled() => {
                    warn!("Shutdown solicitado pelo Windows SCM.");
                }
            }
        });

        status_handle
            .set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Stopped,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })
            .unwrap();
    }
}
