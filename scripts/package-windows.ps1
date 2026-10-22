param([Parameter(Mandatory = $true)][ValidatePattern('^\d+\.\d+\.\d+$')][string]$Version)
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$compiler = Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'
if (!(Test-Path $compiler)) { throw 'Install Inno Setup 6 before packaging.' }
if (!(Test-Path (Join-Path $repo 'target\release\remain.exe'))) { throw 'Build the release executable first.' }
if (!(Test-Path (Join-Path $repo 'site\index.html'))) { throw 'Build the documentation with bash scripts/build-docs.sh first.' }
New-Item -ItemType Directory -Force (Join-Path $repo 'dist') | Out-Null
& $compiler "/DAppVersion=$Version" "/DRepoRoot=$repo" (Join-Path $repo 'packaging\windows\remain.iss')
if ($LASTEXITCODE -ne 0) { throw 'Inno Setup compilation failed.' }
