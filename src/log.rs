use std::path::PathBuf;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*};

pub fn init_logger(path: PathBuf) -> WorkerGuard {
    let file_appender = tracing_appender::rolling::daily(path, "service.log");
    // Esse guard é necessário pra conseguir salvar os logs nos arquivos.
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // file layer é mais potente, com mais dados e em formato json
    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_current_span(true)
        .with_span_list(true)
        .with_line_number(true)
        .with_file(true)
        .with_thread_names(true)
        .with_ansi(false)
        .with_filter(LevelFilter::DEBUG);

    #[cfg(target_os = "windows")]
    let system_layer =
        win_event_log::WinLayer::new("MeuServico").with_filter(filter::LevelFilter::WARN);

    #[cfg(target_os = "linux")]
    let system_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_target(false);

    // Pega o env filter a partir da variável RUST_LOG ou seta o nível de INFO por padrão
    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    // Registra tudo no tracing e inicializa
    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(system_layer)
        .init();

    guard
}

/// Vibecoded total isso aqui, revisar pra entender melhor depois, mas basicamente cria um cara
/// compatível com a lib mais poderosa de logging que eu já vi, a tracing, pra quando chamar por
/// exemplo:
///
/// `info!("TARARA")` no windows, ele enviar tanto pro arquivo configurado, quanto pro report lá do
/// Windows
#[cfg(target_os = "windows")]
mod win_event_log {
    use tracing::{Event, Subscriber};
    use tracing_subscriber::layer::{Context, Layer};

    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::EventLog::{
        EVENTLOG_ERROR_TYPE, EVENTLOG_INFORMATION_TYPE, EVENTLOG_WARNING_TYPE,
        RegisterEventSourceW, ReportEventW,
    };

    #[derive(Default)]
    pub struct EventLogVisitor {
        pub message: String,
        pub fields: String,
    }

    impl tracing::field::Visit for EventLogVisitor {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            if field.name() == "message" {
                self.message = format!("{:?}", value);
            } else {
                if !self.fields.is_empty() {
                    self.fields.push_str(", ");
                }
                self.fields
                    .push_str(&format!("{}={:?}", field.name(), value));
            }
        }
    }

    pub struct WinLayer {
        handle: HANDLE,
    }

    unsafe impl Send for WinLayer {}
    unsafe impl Sync for WinLayer {}

    impl WinLayer {
        pub fn new(source_name: &str) -> Self {
            // Converte o nome da fonte para UTF-16 (wide string)
            let source_utf16: Vec<u16> = source_name
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let handle = unsafe {
                RegisterEventSourceW(None, windows::core::PCWSTR(source_utf16.as_ptr())).unwrap()
            };
            Self { handle }
        }
    }

    impl<S: Subscriber> Layer<S> for WinLayer {
        fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
            let mut visitor = EventLogVisitor::default();
            event.record(&mut visitor);

            let event_type = match *event.metadata().level() {
                tracing::Level::ERROR => EVENTLOG_ERROR_TYPE,
                tracing::Level::WARN => EVENTLOG_WARNING_TYPE,
                _ => EVENTLOG_INFORMATION_TYPE,
            };

            let full_message = if visitor.fields.is_empty() {
                visitor.message
            } else {
                format!("{} [{}]", visitor.message, visitor.fields)
            };

            let msg_utf16: Vec<u16> = full_message
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let lpstrings = [windows::core::PCWSTR::from_raw(msg_utf16.as_ptr())];

            unsafe {
                let _ = ReportEventW(
                    self.handle,      // 1. Handle
                    event_type,       // 2. Tipo
                    0,                // 3. Categoria
                    1,                // 4. ID do Evento
                    None,             // 5. User SID
                    0,                // 6. dwDataSize (u32) - O que faltava!
                    Some(&lpstrings), // 7. lpStrings (Option<&[PCWSTR]>)
                    None,             // 8. lpRawData (Option<*const c_void>)
                );
            }
        }
    }
}
