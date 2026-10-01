// Transmissor: no Windows, abre um WebView2 oculto que captura a tela inteira e transmite via WebRTC.
// Sem janela, sem ícone na barra de tarefas, sem seletor de tela e sem aviso de compartilhamento.
// O console fica aberto só para mostrar o link; fechar o console (ou Ctrl+C) encerra a transmissão.
mod server;

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
    let listener = std::net::TcpListener::bind(("0.0.0.0", port))
        .unwrap_or_else(|e| panic!("Não foi possível usar a porta {port}: {e}"));

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
                println!("Fora do Windows não há captura automática: para transmitir, abra http://localhost:{port}/broadcast no Chrome.");
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o app");
}
