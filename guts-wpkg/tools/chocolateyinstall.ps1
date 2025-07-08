$exeName = 'guts.exe'
$toolsDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$installDir = "$env:ChocolateyInstall\lib\guts-wpkg\tools"

New-Item -ItemType Directory -Path $installDir -Force

Copy-Item -Path "$toolsDir\$exeName" -Destination $installDir

# add PATH sys
$targetPath = [IO.Path]::Combine($installDir, $exeName)
Install-ChocolateyShortcut `
  -ShortcutFilePath "$env:ProgramData\chocolatey\bin\guts.cmd" `
  -TargetPath $targetPath

Write-Host "Guts has been installed, thank you for using our application. Visit the Github, and don't forget to check out the wiki and give it a star!"
