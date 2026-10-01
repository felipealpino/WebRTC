// Transmissor: no Windows, abre um WebView2 oculto que captura a tela inteira e transmite via WebRTC.
// Roda em segundo plano: sem console, janela, ícone, seletor de tela ou aviso de compartilhamento.
// O link e os erros vão para transmissao.log, ao lado do .exe. Para parar: stop-stream.ps1.
#![cfg_attr(windows, windows_subsystem = "windows")]

macro_rules! log {
    ($($arg:tt)*) => { crate::write_log(&format!($($arg)*)) };
}

mod server;

fn log_path() -> std::path::PathBuf {
    std::env::current_exe().expect("caminho do .exe").with_file_name("transmissao.log")
}

fn write_log(msg: &str) {
    use std::io::Write;
    println!("{msg}");
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(log_path()) {
        let _ = writeln!(file, "{msg}");
    }
}

// Flags do WebView2 (Chromium):
// - use-fake-ui-for-media-stream: aceita a captura da tela inteira sem seletor nem aviso.
// - AllowWgcScreenCapturer desligado: captura via DXGI, sem a borda amarela do Windows Graphics Capture.
// - WebRtcHideLocalIpsWithMdns desligado: anuncia o IP real na rede local (mais confiável que nomes .local).
// - background/backgrounding: a página oculta não é desacelerada.
// - msWebOOUI, msPdfOOUI, msSmartScreenProtection: padrão do Tauri, mantido porque esta lista o substitui.
#[cfg(windows)]
const BROWSER_ARGS: &str = "--use-fake-ui-for-media-stream \
    --disable-background-timer-throttling --disable-renderer-backgrounding --disable-backgrounding-occluded-windows \
    --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,AllowWgcScreenCapturer,WebRtcHideLocalIpsWithMdns,GetDisplayMediaRequiresUserActivation";

fn main() {
    let config = server::Config::from_env();
    let port = config.port;
    let listener = match std::net::TcpListener::bind(("0.0.0.0", port)) {
        Ok(listener) => listener,
        Err(e) => {
            log!("Não foi possível usar a porta {port} (o app já está rodando?): {e}");
            std::process::exit(1);
        }
    };
    // Só limpa o log depois de garantir a porta, para não apagar o log de outra cópia já rodando.
    let _ = std::fs::remove_file(log_path());

    tauri::Builder::default()
        .setup(move |app| {
            tauri::async_runtime::spawn(server::run(listener, config));

            #[cfg(windows)]
            {
                let url = format!("http://localhost:{port}/broadcast?auto").parse()?;
                tauri::WebviewWindowBuilder::new(app, "transmissor", tauri::WebviewUrl::External(url))
                    .visible(false)
                    .skip_taskbar(true)
                    .additional_browser_args(BROWSER_ARGS)
                    .build()?;
            }
            #[cfg(not(windows))]
            {
                let _ = app;
                log!("Fora do Windows não há captura automática: para transmitir, abra http://localhost:{port}/broadcast no Chrome.");
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o app");
}
