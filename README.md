# Transmissão de tela

Transmite a tela de um PC Windows em tempo real (WebRTC, até 1080p a 30 fps) para quem abrir um link no navegador da rede local.
No PC que transmite, o app roda em segundo plano: não aparece janela, console, seletor de tela nem aviso de compartilhamento.

## Uso no Windows

**1. Instalar ou atualizar.** Abra o PowerShell como administrador (Iniciar, digite `powershell`, botão direito, "Executar como administrador") e rode:

```powershell
irm https://raw.githubusercontent.com/felipealpino/WebRTC/main/install.ps1 | iex
```

O script ([install.ps1](install.ps1)) baixa o app e os scripts `stop-stream.ps1` e `uninstall.ps1` para `C:\webrtc`, libera o app no firewall e instala o WebView2 se faltar. Rode de novo para atualizar.

**2. Transmitir.** Dê duplo clique em `C:\webrtc\webrtc.exe`. O app roda em segundo plano.

O link fica em `C:\webrtc\webrtc.log`, por exemplo `http://192.168.0.50:8080`.

**3. Assistir.** Em qualquer Mac ou Windows da rede, abra o link no Chrome, Edge ou Safari. Duplo clique no vídeo põe em tela cheia.

**4. Parar.** Em `C:\webrtc`, clique com o botão direito em `stop-stream.ps1` e escolha "Executar com o PowerShell".

**Iniciar com o Windows.** A instalação já cria um atalho na pasta Inicializar do usuário, então o app sobe sozinho em segundo plano sempre que você entra no Windows (no login, não antes, porque precisa da área de trabalho para capturar a tela). Para desligar isso sem desinstalar, apague o atalho:

```powershell
Remove-Item "$([Environment]::GetFolderPath('Startup'))\webrtc.lnk"
```

**Desinstalar.** Em `C:\webrtc`, clique com o botão direito em `uninstall.ps1` e escolha "Executar com o PowerShell". O script remove a pasta, o atalho de inicialização, as regras de firewall e os dados do app.

## Configuração (opcional)

Defina as variáveis no PowerShell e abra o app pelo mesmo PowerShell:

```powershell
$env:PORT = "8080"; $env:FPS = "30"; $env:BITRATE_MBPS = "8"; C:\webrtc\webrtc.exe
```

- `PORT`: porta HTTP (padrão `8080`).
- `FPS`: quadros por segundo (padrão `30`).
- `BITRATE_MBPS`: taxa máxima por espectador (padrão `8`).
- `ICE_SERVERS`: STUN/TURN para uso fora da rede local, por exemplo `[{"urls":"stun:stun.l.google.com:19302"}]`.

## Problemas comuns

- **`Erro ao capturar a tela…` no `webrtc.log`:** a mensagem diz o motivo. O app tenta de novo a cada 5 s.
- **Quem assiste fica em "Conectando…":** confira se os PCs estão na mesma rede e se o firewall foi liberado (rode o `install.ps1` de novo).
- **Quem assiste fica em "Aguardando transmissão…":** o app não está rodando, ou a captura falhou. Veja o `webrtc.log`.
- **`Não foi possível usar a porta 8080` no `webrtc.log`:** o app já está rodando, ou outro programa usa a porta. Rode o `stop-stream.ps1` ou use `$env:PORT`.

## Estrutura

```
public/
  index.html           página de quem assiste
  broadcast.html       captura a tela e abre uma conexão WebRTC por espectador
src-tauri/
  src/main.rs          app Tauri: no Windows, abre um WebView2 oculto em /broadcast?auto
  src/server.rs        servidor HTTP + WebSocket (sinalização WebRTC)
  tauri.conf.json      configuração do Tauri
install.ps1            instalador para Windows
stop-stream.ps1        para a transmissão
uninstall.ps1          remove o app e tudo que a instalação criou
.github/workflows/     build do .exe a cada push na main, publicado na release "latest"
```

Como funciona:

- O WebView2 oculto captura a tela. Flags do WebView2 em `main.rs` fazem a captura ser aceita sem seletor nem aviso.
- O servidor só passa a sinalização. O vídeo vai direto do transmissor para cada espectador, com H.264 preferido. Nada é gravado.
- Só o próprio PC (localhost) pode transmitir. Qualquer um na rede pode assistir.

## Desenvolvimento

Pré-requisito: [Rust](https://rustup.rs).

```bash
cd src-tauri && cargo run
```

Fora do Windows, o app só sobe o servidor. Para testar a transmissão, abra `http://localhost:8080/broadcast` no Chrome e clique em "Compartilhar tela".

Cada push na `main` gera um `.exe` novo na release `latest`. O `install.ps1` baixa esse `.exe`.

## Limites

- Cada espectador recebe um fluxo codificado à parte. Funciona bem para 5 a 10 pessoas. Para mais gente, use um SFU (por exemplo, MediaMTX no Raspberry Pi).
- Com mais de um monitor, a captura pega a área de trabalho inteira.
- Só vídeo, sem áudio.
- Uso remoto: coloque o servidor atrás de HTTPS (as páginas já usam `wss://`) e configure `ICE_SERVERS` com um TURN, por exemplo coturn no Raspberry Pi.
