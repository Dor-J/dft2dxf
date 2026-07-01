$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$Pattern = 'use std::any::Any|use std::ffi::c_void|\bunsafe\b|\*const\b|\*mut\b'
$Paths = @('crates', 'fuzz')
$Matches = @()

foreach ($Path in $Paths) {
  Get-ChildItem -Path $Path -Filter '*.rs' -Recurse -File | ForEach-Object {
    $Hits = Select-String -Path $_.FullName -Pattern $Pattern
    if ($Hits) {
      $Matches += $Hits
    }
  }
}

if ($Matches.Count -gt 0) {
  $Matches | ForEach-Object { Write-Host $_ }
  Write-Host ''
  Write-Host 'error: forbidden Rust constructs found (see CONTRIBUTING.md safe Rust policy)'
  exit 1
}

Write-Host 'safe Rust policy: OK'
