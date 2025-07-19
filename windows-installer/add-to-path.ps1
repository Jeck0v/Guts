$InstallPath = "$env:ProgramFiles\Guts"
$CurrentMachinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")

$logFile = "$env:Temp\guts_install_log.txt"
"Current MACHINE PATH: $CurrentMachinePath" | Out-File -FilePath $logFile -Append
"InstallPath: $InstallPath" | Out-File -FilePath $logFile -Append

if ($CurrentMachinePath -notmatch [Regex]::Escape($InstallPath)) {
    $NewMachinePath = "$CurrentMachinePath;$InstallPath"
    [Environment]::SetEnvironmentVariable("Path", $NewMachinePath, "Machine")
    "Machine PATH updated successfully." | Out-File -FilePath $logFile -Append
} else {
    "Machine PATH already contains install path." | Out-File -FilePath $logFile -Append
}
