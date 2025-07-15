$exeName = 'guts.exe'
$toolsDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$installDir = "$env:ChocolateyInstall\lib\guts-wpkg\tools"

Remove-Item -Path "$installDir\$exeName" -Force -ErrorAction SilentlyContinue
Remove-Item "$env:ProgramData\chocolatey\bin\guts.cmd" -Force -ErrorAction SilentlyContinue
Write-Host "Guts has been uninstalled, please come back and see us, we miss you already!"
