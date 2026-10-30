#requires -Version 5.1
<#
.SYNOPSIS
    Собирает минимальный релиз TAniCraft и кладёт в dist только готовый exe.

.DESCRIPTION
    Каталог target\release — это кэш сборки Cargo: в нём лежат rlib зависимостей,
    build-скрипты и PDB, поэтому он занимает сотни мегабайт. Для раздачи нужен
    только target\release\tanicraft.exe. Этот скрипт собирает профиль release
    (opt-level=z, fat LTO, panic=abort, strip) и копирует один файл в dist\,
    при необходимости упаковывая его в zip.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\package.ps1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\package.ps1 -NoZip
#>
[CmdletBinding()]
param(
    [switch]$NoZip
)

$ErrorActionPreference = 'Stop'
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
    Write-Host 'Building release...'
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build --release failed with exit code $LASTEXITCODE"
    }

    $exe = Join-Path $repoRoot 'target\release\tanicraft.exe'
    if (-not (Test-Path $exe)) {
        throw "Release executable not found: $exe"
    }

    $dist = Join-Path $repoRoot 'dist'
    New-Item -ItemType Directory -Path $dist -Force | Out-Null

    $out = Join-Path $dist 'TAniCraft.exe'
    Copy-Item -Path $exe -Destination $out -Force

    # MIT требует сохранять уведомление об авторстве при распространении,
    # поэтому лицензия едет рядом с exe и попадает в архив.
    $license = Join-Path $repoRoot 'LICENSE'
    if (Test-Path $license) {
        Copy-Item -Path $license -Destination (Join-Path $dist 'LICENSE.txt') -Force
    }
    else {
        Write-Warning "LICENSE not found; the distributed build will lack the required notice."
    }

    $mb = [math]::Round((Get-Item $out).Length / 1MB, 2)
    Write-Host ("dist\TAniCraft.exe  {0} MB" -f $mb)

    if (-not $NoZip) {
        $zip = Join-Path $dist 'TAniCraft-win64.zip'
        Remove-Item -Path $zip -Force -ErrorAction SilentlyContinue
        $items = @($out)
        if (Test-Path $license) {
            $items += (Join-Path $dist 'LICENSE.txt')
        }
        Compress-Archive -Path $items -DestinationPath $zip -CompressionLevel Optimal
        $zipMb = [math]::Round((Get-Item $zip).Length / 1MB, 2)
        Write-Host ("dist\TAniCraft-win64.zip  {0} MB" -f $zipMb)
    }
}
finally {
    Pop-Location
}
