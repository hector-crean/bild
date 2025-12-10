# PowerShell script to remove target directories from Rust projects
# Usage: .\clean-targets.ps1 [directory] [--dry-run]

param(
    [string]$Directory = ".",
    [switch]$DryRun = $false
)

$ErrorActionPreference = "Stop"

# Get all target directories
$targetDirs = Get-ChildItem -Path $Directory -Directory -Recurse -Filter "target" -ErrorAction SilentlyContinue | 
    Where-Object { 
        # Only match target directories that are actual Rust build directories
        # (they should have a parent directory with Cargo.toml)
        $parent = $_.Parent.FullName
        Test-Path (Join-Path $parent "Cargo.toml")
    }

if ($targetDirs.Count -eq 0) {
    Write-Host "No target directories found in $Directory" -ForegroundColor Yellow
    exit 0
}

Write-Host "Found $($targetDirs.Count) target directory/directories:" -ForegroundColor Cyan
foreach ($dir in $targetDirs) {
    $size = (Get-ChildItem -Path $dir.FullName -Recurse -ErrorAction SilentlyContinue | 
             Measure-Object -Property Length -Sum).Sum
    $sizeGB = [math]::Round($size / 1GB, 2)
    Write-Host "  $($dir.FullName) ($sizeGB GB)" -ForegroundColor Gray
}

$totalSize = ($targetDirs | ForEach-Object {
    (Get-ChildItem -Path $_.FullName -Recurse -ErrorAction SilentlyContinue | 
     Measure-Object -Property Length -Sum).Sum
} | Measure-Object -Sum).Sum
$totalSizeGB = [math]::Round($totalSize / 1GB, 2)

Write-Host "`nTotal size: $totalSizeGB GB" -ForegroundColor Cyan

if ($DryRun) {
    Write-Host "`n[DRY RUN] Would remove the above directories" -ForegroundColor Yellow
    exit 0
}

$confirmation = Read-Host "`nRemove all target directories? (y/N)"
if ($confirmation -ne "y" -and $confirmation -ne "Y") {
    Write-Host "Cancelled." -ForegroundColor Yellow
    exit 0
}

Write-Host "`nRemoving target directories..." -ForegroundColor Green
$removed = 0
$errors = 0

foreach ($dir in $targetDirs) {
    try {
        Remove-Item -Path $dir.FullName -Recurse -Force -ErrorAction Stop
        Write-Host "  Removed: $($dir.FullName)" -ForegroundColor Green
        $removed++
    }
    catch {
        Write-Host "  Error removing $($dir.FullName): $_" -ForegroundColor Red
        $errors++
    }
}

Write-Host "`nDone! Removed $removed directory/directories" -ForegroundColor Green
if ($errors -gt 0) {
    Write-Host "Encountered $errors error(s)" -ForegroundColor Red
}

