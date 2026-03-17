$ErrorActionPreference = 'Stop'

$ZENITH_REPO = "neevets/zenith"
$ZENITH_BIN = "zenith.exe"
$ZENITH_QUIET = $false
$ZENITH_YES = $false
$ZENITH_INSTALL_DIR = ""

function Say($msg) {
    if (-not $ZENITH_QUIET) { Write-Host $msg }
}

function Err($msg) {
    Write-Error "error: $msg" -ErrorAction Continue
}

function Usage() {
    Write-Host @"
Zenith installer (Windows)

Usage: .\install.ps1 [OPTIONS]

Options:
  -q, --quiet        Reduce output
  -y                 Skip confirmation prompt
      --to <DIR>     Install directory
  -h, --help         Show this help
"@
}

function Get-Target() {
    $arch = $env:PROCESSOR_ARCHITECTURE.ToLower()
    $cpu = "x86_64"
    if ($arch -eq "arm64") { $cpu = "arm64" }
    return "windows-$cpu"
}

function Main {
    param($args)

    $i = 0
    while ($i -lt $args.Length) {
        switch ($args[$i]) {
            "-q" { $ZENITH_QUIET = $true }
            "--quiet" { $ZENITH_QUIET = $true }
            "-y" { $ZENITH_YES = $true }
            "--to" { 
                $i++
                $ZENITH_INSTALL_DIR = $args[$i] 
            }
            "-h" { Usage; return }
            "--help" { Usage; return }
        }
        $i++
    }

    $target = Get-Target
    $cpu = $target.Split("-")[1]
    
    if ($ZENITH_INSTALL_DIR -eq "") {
        $installDir = Join-Path $HOME ".local\bin"
    } else {
        $installDir = $ZENITH_INSTALL_DIR
    }

    if (-not $ZENITH_YES) {
        $choice = Read-Host "Install Zenith to '$installDir'? [y/N]"
        if ($choice -notmatch "[yY]") { 
            Write-Host "Aborted."; return 
        }
    }

    $tmpDir = Join-Path $env:TEMP "zenith-$(Get-Random)"
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null
    $zipFile = Join-Path $tmpDir "zenith.tar.gz"

    $candidates = @("zenith-windows-$cpu.tar.gz", "zenith-windows-x86_64.tar.gz")
    $found = $false

    foreach ($artifact in $candidates) {
        $url = "https://github.com/$ZENITH_REPO/releases/latest/download/$artifact"
        try {
            Say "target: $target"
            Say "downloading: $url"
            Invoke-WebRequest -Uri $url -OutFile $zipFile -ErrorAction Stop
            $found = $true
            break
        } catch {
            continue
        }
    }

    if (-not $found) {
        Err "No release artifact available for $target"
        return
    }

    try {
        $oldDir = Get-Location
        Set-Location $tmpDir
        tar -xzf $zipFile
        Set-Location $oldDir
    } catch {
        Err "Failed to extract archive."
        return
    }

    $sourceBin = Get-ChildItem -Path $tmpDir -Include "zenith.exe" -Recurse | Select-Object -First 1
    if (-not $sourceBin) {
        Err "Binary 'zenith.exe' not found."
        return
    }

    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    }

    $dest = Join-Path $installDir "zenith.exe"
    Copy-Item $sourceBin.FullName -Destination $dest -Force
    Remove-Item $tmpDir -Recurse -Force

    Say "installed: $dest"
    
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$installDir*") {
        $currentPath = "$currentPath;$installDir"
        [Environment]::SetEnvironmentVariable("Path", $currentPath, "User")
        Say "PATH updated."
    }

    Say "run: zenith --version"
}

Main $args
