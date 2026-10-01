# Transmissão de tela

Transmite a tela de um PC Windows em tempo real (WebRTC, até 1080p a 30 fps) para quem abrir um link no navegador da rede local.

## Instalar

Abra o PowerShell como administrador e rode:

```powershell
irm https://raw.githubusercontent.com/felipealpino/WebRTC/main/install.ps1 | iex
```

Isso instala em `C:\webrtc`, libera o firewall, já inicia o app em segundo plano e faz ele subir sozinho a cada login. Rode de novo para atualizar.

O link para assistir fica em `C:\webrtc\webrtc.log` (ex.: `http://192.168.0.50:8080`). Abra num navegador de qualquer PC da rede.
