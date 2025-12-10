# PowerShell script to analyze disk space usage
# Usage: .\analyze-disk-space.ps1 [directory] [--top N] [--min-size MB]

param(
    [string]$Directory = ".",
    [int]$Top = 20,
    [int]$MinSizeMB = 10
)

$ErrorActionPreference = "Continue"

Write-Host "Analyzing disk space usage..." -ForegroundColor Cyan
Write-Host "Directory: $((Resolve-Path $Directory).Path)" -ForegroundColor Gray
Write-Host ""

# Get all directories and their sizes
$results = @()

Write-Host "Scanning directories (this may take a while)..." -ForegroundColor Yellow

Get-ChildItem -Path $Directory -Directory -Recurse -ErrorAction SilentlyContinue | ForEach-Object {
    $dir = $_
    try {
        $size = (Get-ChildItem -Path $dir.FullName -Recurse -File -ErrorAction SilentlyContinue | 
                 Measure-Object -Property Length -Sum -ErrorAction SilentlyContinue).Sum
        
        if ($size -gt ($MinSizeMB * 1MB)) {
            $results += [PSCustomObject]@{
                Path = $dir.FullName
                Size = $size
                SizeGB = [math]::Round($size / 1GB, 2)
                SizeMB = [math]::Round($size / 1MB, 2)
            }
        }
    }
    catch {
        # Skip directories we can't access
    }
}

# Sort by size and get top N
$topDirs = $results | Sort-Object -Property Size -Descending | Select-Object -First $Top

Write-Host "`nTop $Top directories by size:" -ForegroundColor Cyan
Write-Host ("=" * 80) -ForegroundColor Gray

$rank = 1
foreach ($dir in $topDirs) {
    $relativePath = $dir.Path.Replace((Resolve-Path $Directory).Path, ".")
    if ($relativePath.Length -gt 60) {
        $relativePath = "..." + $relativePath.Substring($relativePath.Length - 57)
    }
    
    $color = if ($dir.SizeGB -gt 1) { "Red" } 
             elseif ($dir.SizeMB -gt 100) { "Yellow" } 
             else { "White" }
    
    Write-Host ("{0,3}. {1,-60} {2,10} GB ({3,10} MB)" -f $rank, $relativePath, $dir.SizeGB, $dir.SizeMB) -ForegroundColor $color
    $rank++
}

# Summary
$totalSize = ($results | Measure-Object -Property Size -Sum).Sum
$totalSizeGB = [math]::Round($totalSize / 1GB, 2)
$totalSizeMB = [math]::Round($totalSize / 1MB, 2)

Write-Host ("`n" + "=" * 80) -ForegroundColor Gray
Write-Host "Total size analyzed: $totalSizeGB GB ($totalSizeMB MB)" -ForegroundColor Cyan
Write-Host "Directories found: $($results.Count)" -ForegroundColor Cyan

# Check for common large directories
Write-Host "`nChecking for common large directories..." -ForegroundColor Cyan
$commonDirs = @(
    @{ Name = "target (Rust)"; Pattern = "target" },
    @{ Name = "node_modules"; Pattern = "node_modules" },
    @{ Name = ".git"; Pattern = ".git" },
    @{ Name = "__pycache__"; Pattern = "__pycache__" },
    @{ Name = ".venv"; Pattern = ".venv" },
    @{ Name = "venv"; Pattern = "venv" },
    @{ Name = ".cargo"; Pattern = ".cargo" }
)

foreach ($common in $commonDirs) {
    $dirs = Get-ChildItem -Path $Directory -Directory -Recurse -Filter $common.Pattern -ErrorAction SilentlyContinue
    if ($dirs) {
        $total = ($dirs | ForEach-Object {
            (Get-ChildItem -Path $_.FullName -Recurse -File -ErrorAction SilentlyContinue | 
             Measure-Object -Property Length -Sum -ErrorAction SilentlyContinue).Sum
        } | Measure-Object -Sum).Sum
        
        if ($total -gt 0) {
            $sizeGB = [math]::Round($total / 1GB, 2)
            Write-Host ("  {0,-20} {1,3} directories, {2,10} GB" -f $common.Name, $dirs.Count, $sizeGB) -ForegroundColor Yellow
        }
    }
}

