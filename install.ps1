# Instala ou atualiza o transmissor de tela em C:\transmissao-tela.
# Rode no PowerShell como administrador:
#   irm https://raw.githubusercontent.com/felipealpino/WebRTC/main/install.ps1 | iex
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = 'Tls12'

$dir = 'C:\transmissao-tela'
$exe = Join-Path $dir 'transmissao-tela.exe'

# WebView2: já vem no Windows 10 atualizado; instala só se faltar.
$webview2 = '{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
$hasWebView2 = (Test-Path "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\$webview2") -or
               (Test-Path "HKCU:\Software\Microsoft\EdgeUpdate\Clients\$webview2")
if (-not $hasWebView2) {
    Write-Host 'Instalando WebView2...'
    winget install --id Microsoft.EdgeWebView2Runtime -e --silent --accept-source-agreements --accept-package-agreements
}

Write-Host 'Baixando o app...'
New-Item -ItemType Directory -Force $dir | Out-Null
Invoke-WebRequest 'https://github.com/felipealpino/WebRTC/releases/download/latest/transmissao-tela.exe' -OutFile $exe

if (-not (Get-NetFirewallRule -DisplayName 'Transmissao de tela' -ErrorAction SilentlyContinue)) {
    New-NetFirewallRule -DisplayName 'Transmissao de tela' -Direction Inbound -Program $exe -Action Allow | Out-Null
}

Write-Host "Pronto. Para transmitir, rode: $exe"
