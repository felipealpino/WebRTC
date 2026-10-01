# Transmissão de tela

App (Tauri + WebRTC) que transmite a tela de um PC Windows em tempo real, até 1080p a 30 fps, para quem abrir um link no navegador da rede local.
No PC que transmite não aparece janela, ícone na barra de tarefas, seletor de tela nem aviso de compartilhamento: só um console com o link.

O `.exe` para Windows é compilado automaticamente pelo GitHub a cada push na `main` e publicado na release [`latest`](https://github.com/felipealpino/WebRTC/releases/tag/latest). No Windows não é preciso instalar Rust, Node nem nada para compilar.

---

## Passo a passo no Windows (PC que transmite)

Abra o **PowerShell como administrador** (Iniciar, digite `powershell`, clique com o botão direito e escolha "Executar como administrador") e siga os passos.

### 1. Instalar o WebView2 e o GitHub CLI (uma vez)

O WebView2 costuma já vir no Windows 10; se já estiver instalado, o comando só avisa isso.

```powershell
winget install --id Microsoft.EdgeWebView2Runtime -e
```

```powershell
winget install --id GitHub.cli -e
```

**Feche e abra o PowerShell de novo** (para o `gh` entrar no PATH) e faça login no GitHub. Escolha *GitHub.com*, *HTTPS* e *Login with a web browser*:

```powershell
gh auth login
```

### 2. Liberar a porta no firewall (uma vez)

```powershell
New-NetFirewallRule -DisplayName "Transmissao de tela" -Direction Inbound -Protocol TCP -LocalPort 8080 -Action Allow
```

### 3. Baixar o app (repita para atualizar)

```powershell
gh release download latest -R felipealpino/WebRTC -p transmissao-tela.exe -D C:\transmissao-tela --clobber
```

> Sem o `gh`: abra https://github.com/felipealpino/WebRTC/releases/tag/latest logado no GitHub, baixe o `transmissao-tela.exe` e rode `Unblock-File` nele. Se aparecer "O Windows protegeu o computador", clique em **Mais informações** e depois em **Executar assim mesmo**.

### 4. Transmitir

```powershell
C:\transmissao-tela\transmissao-tela.exe
```

O console mostra algo assim:

```
Para assistir, abra no navegador: http://192.168.0.50:8080
Transmissão iniciada.
[transmissor] Capturando 1920x1080 @ 30 fps — 0 assistindo
```

- Pode minimizar o console. Para **parar**, use **Ctrl+C** ou feche o console.
- Também dá para abrir com duplo clique em `C:\transmissao-tela\transmissao-tela.exe`.

### 5. Quem assiste (Mac ou Windows)

Abra o link do console (`http://IP-DO-PC:8080`) no Chrome, Edge ou Safari. Dê um **duplo clique no vídeo** para tela cheia. Não precisa instalar nada.

---

## Configuração (opcional)

São variáveis de ambiente, definidas no PowerShell antes de rodar:

```powershell
$env:PORT = "8080"; $env:FPS = "30"; $env:BITRATE_MBPS = "8"; C:\transmissao-tela\transmissao-tela.exe
```

| Variável | Padrão | Para que serve |
|---|---|---|
| `PORT` | `8080` | Porta HTTP (se mudar, libere a nova porta no firewall) |
| `FPS` | `30` | Quadros por segundo |
| `BITRATE_MBPS` | `8` | Taxa máxima por espectador |
| `ICE_SERVERS` | vazio | STUN/TURN para uso fora da rede local, por exemplo `[{"urls":"stun:stun.l.google.com:19302"}]` |

## Problemas comuns

| Sintoma | O que fazer |
|---|---|
| O console mostra `Erro ao capturar a tela…` | Copie a mensagem: ela diz o motivo. O app tenta de novo a cada 5 s. |
| Quem assiste fica em "Conectando…" | Confira se está na mesma rede e se a regra do passo 2 foi criada (`Get-NetFirewallRule -DisplayName "Transmissao de tela"`). |
| Fica em "Aguardando transmissão…" | O app não está rodando, ou a captura falhou (veja o console). |
| `Não foi possível usar a porta 8080` | Já existe outro programa (ou outra cópia do app) usando a porta. Feche-o ou use `$env:PORT`. |
| Aparece uma borda amarela na tela | É a captura do Windows; me avise que ajustamos as flags em `src-tauri/src/main.rs`. |

---

## Como funciona

- `src-tauri/src/main.rs`: no Windows, abre um WebView2 **oculto** em `http://localhost:8080/broadcast?auto`. Flags do WebView2 fazem a captura da tela inteira ser aceita automaticamente, sem seletor nem aviso.
- `src-tauri/src/server.rs`: servidor HTTP e WebSocket (sinalização WebRTC). Só o próprio PC (localhost) pode transmitir; qualquer um na rede pode assistir.
- `public/broadcast.html`: captura a tela e abre uma conexão WebRTC por espectador, com H.264 preferido.
- `public/index.html`: página de quem assiste.
- O vídeo vai direto do transmissor para cada espectador. Não passa pelo servidor e nada é gravado.

## Desenvolvimento

### No Mac

No macOS o app só sobe o servidor (sem captura automática). Para testar a transmissão, abra `http://localhost:8080/broadcast` no Chrome e clique em "Compartilhar tela".

```bash
cd src-tauri && cargo run
```

### Compilar no próprio Windows (opcional)

Só é necessário se não quiser esperar o build do GitHub. No PowerShell como administrador:

```powershell
winget install --id Rustlang.Rustup -e
```

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

Reabra o PowerShell e rode:

```powershell
gh repo clone felipealpino/WebRTC; cd WebRTC\src-tauri; cargo build --release
```

O app sai em `src-tauri\target\release\transmissao-tela.exe`.

## Limites e próximos passos

- Cada espectador recebe um fluxo codificado à parte. Para algo como 5 a 10 pessoas isso funciona bem. Para muito mais gente, o caminho é um SFU (por exemplo, o MediaMTX no Raspberry Pi) repassando um único fluxo.
- Com mais de um monitor, a captura automática pega a área de trabalho inteira.
- Uso remoto: coloque o servidor atrás de HTTPS (as páginas já usam `wss://` automaticamente) e configure `ICE_SERVERS` com um TURN, que pode ser o coturn no Raspberry Pi.
- Só transmite vídeo, sem áudio.
