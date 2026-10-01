# Remove o transmissor de tela e tudo que a instalação criou.
# Clique com o botão direito e escolha "Executar com o PowerShell" (pede permissão de administrador).
$identity = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $identity.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Start-Process powershell -Verb RunAs -ArgumentList "-ExecutionPolicy Bypass -File `"$PSCommandPath`""
    return
}

$dir = 'C:\transmissao-tela'

Get-Process transmissao-tela -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 1

Remove-NetFirewallRule -DisplayName 'Transmissao de tela', 'Transmissao de tela (WebRTC)' -ErrorAction SilentlyContinue
Remove-MpPreference -ExclusionPath $dir, "$dir\transmissao-tela.exe" -ErrorAction SilentlyContinue

# Sai da pasta antes de apagá-la.
Set-Location $env:TEMP
Remove-Item -Recurse -Force $dir, "$env:LOCALAPPDATA\com.felipealpino.transmissao-tela" -ErrorAction SilentlyContinue

Write-Host 'Transmissor de tela removido.'
Start-Sleep 3
