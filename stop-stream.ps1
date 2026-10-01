# Para a transmissão. Clique com o botão direito e escolha "Executar com o PowerShell".
$app = Get-Process transmissao-tela -ErrorAction SilentlyContinue
if ($app) {
    $app | Stop-Process -Force
    Write-Host 'Transmissão parada.'
} else {
    Write-Host 'A transmissão não estava rodando.'
}
Start-Sleep 2
