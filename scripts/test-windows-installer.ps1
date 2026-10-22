# Run only in CI or a disposable Windows account: this exercises Start-menu registration.
param([Parameter(Mandatory = $true)][string]$Installer)
$ErrorActionPreference = 'Stop'
$installerPath = (Get-Item $Installer).FullName
$destination = Join-Path ([IO.Path]::GetTempPath()) "remain-install-$([Guid]::NewGuid())"

function Invoke-Setup([string]$Path, [string[]]$ExtraArguments = @()) {
    $arguments = @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') + $ExtraArguments
    $process = Start-Process -FilePath $Path -ArgumentList $arguments -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw "Setup failed with exit code $($process.ExitCode): $Path" }
}

Invoke-Setup $installerPath @("/DIR=`"$destination`"")
foreach ($file in @('remain.exe', 'LICENSE', 'guide\index.html', 'guide\api\remain_core\index.html')) {
    if (!(Test-Path (Join-Path $destination $file))) { throw "Installed file is missing: $file" }
}
Add-Type -AssemblyName System.Drawing
$icon = [System.Drawing.Icon]::ExtractAssociatedIcon("$destination\remain.exe")
if ($null -eq $icon) { throw 'Installed executable has no icon' }
$icon.Dispose()
$shortcut = Join-Path ([Environment]::GetFolderPath('Programs')) 'REMAIN.lnk'
if (!(Test-Path $shortcut)) { throw 'Start-menu shortcut is missing' }
Invoke-Setup "$destination\unins000.exe"
if ((Test-Path "$destination\remain.exe") -or (Test-Path $shortcut)) {
    throw 'Uninstall left the executable or Start-menu shortcut behind'
}
Write-Output 'Windows installation, bundled documentation, icon, shortcut, and uninstall passed.'
