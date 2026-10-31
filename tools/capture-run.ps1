# capture-run.ps1 - one Seihai run with a packet capture, packed for sending.
# Windows version of tools/capture-run.sh; same options and output.
#
# Usage, from the seihai folder:
#   .\tools\capture-run.ps1 [--target HOST:PORT] [--version V] [--account A]
#       [--password-file F] [--seihai PATH] [--out DIR] [-- extra seihai args]
# Defaults: listserver.graal.in:14922, 2.22, guest, target\release\seihai.exe, _debug.
#
# The password comes from --password-file, else GRAAL_PW in the environment,
# else a hidden prompt. It reaches Seihai only through --password-env GRAAL_PW.
#
# Capture: Wireshark's dumpcap when installed (no admin needed with Npcap; it
# writes a classic pcap that `seihai capture show` reads). Otherwise the
# built-in pktmon, which needs an elevated PowerShell and writes pcapng (open
# that in Wireshark). Set SEIHAI_CAPTURE_IF to pick the dumpcap interface;
# the default is the one that carries the default route.
#
# Keep a public run to one short login: say hi once, walk through one door, quit.
#
# No param block on purpose: PowerShell would take --target as its own syntax.

$Target = 'listserver.graal.in:14922'
$Version = '2.22'
$Account = 'guest'
$PasswordFile = $null
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$Seihai = Join-Path $repo 'target\release\seihai.exe'
$OutRoot = '_debug'
$Extra = @()

$rest = @($args)
for ($i = 0; $i -lt $rest.Count; $i++) {
    $a = [string]$rest[$i]
    switch ($a) {
        '--target'        { $Target = $rest[++$i] }
        '--version'       { $Version = $rest[++$i] }
        '--account'       { $Account = $rest[++$i] }
        '--password-file' { $PasswordFile = $rest[++$i] }
        '--seihai'        { $Seihai = $rest[++$i] }
        '--out'           { $OutRoot = $rest[++$i] }
        '--'              { $Extra = @($rest[($i + 1)..($rest.Count - 1)] | Where-Object { $_ -ne $null }); $i = $rest.Count }
        { $_ -in '-h', '--help' } { Get-Content $PSCommandPath -TotalCount 19 | ForEach-Object { $_ -replace '^# ?', '' }; exit 0 }
        # PowerShell drops a bare -- when the script is run from a
        # PowerShell prompt, so anything not ours goes to Seihai too.
        default           { $Extra += $a }
    }
}

if (-not (Test-Path $Seihai)) { Write-Error "no seihai at $Seihai; run cargo build --release first"; exit 1 }
$Seihai = (Resolve-Path $Seihai).Path
$hostName, $port = $Target -split ':', 2
if (-not $port) { Write-Error 'target must be HOST:PORT'; exit 2 }

# Listserver and gserver ports: the Graal range, plus the target's port
# (a few servers, such as Jojo's on 30903, sit outside it). dumpcap takes the
# range; pktmon has no port ranges, so it gets the ports seen so far.
$PortRange = '14800-14999'
$Ports = @(14921, 14922, 14923, 14801, 14802, 14842, 14900, 14912, 14913, 14917)
if ($Ports -notcontains [int]$port) { $Ports += [int]$port }

# Output folder readable only by this user (umask 077 / chmod 700).
New-Item -ItemType Directory -Force -Path $OutRoot | Out-Null
$runName = 'run-' + (Get-Date -Format 'yyyyMMdd-HHmmss')
$out = Join-Path (Resolve-Path $OutRoot).Path $runName
New-Item -ItemType Directory -Path $out | Out-Null
& icacls $out /inheritance:r /grant:r "$($env:USERNAME):(OI)(CI)F" | Out-Null

# 6-second TCP connect test.
$tcp = New-Object Net.Sockets.TcpClient
try {
    $ok = $tcp.ConnectAsync($hostName, [int]$port).Wait(6000)
    $portCheck = if ($ok -and $tcp.Connected) { 'open' } else { 'no answer in 6 s' }
} catch {
    $portCheck = "failed ($($_.Exception.InnerException.Message))"
} finally { $tcp.Close() }

$sha = (& git -C $repo rev-parse --short HEAD 2>$null)
@(
    "date: $(Get-Date -Format o)"
    "git: $sha"
    "seihai: $Seihai"
    "target: $Target"
    "version: $Version"
    "account: $Account"
    "port check: $portCheck"
    "capture ports: dumpcap $PortRange and $port; pktmon $($Ports -join ' ')"
) | Out-File -Encoding utf8 (Join-Path $out '00_run_info.txt')
Write-Host "output: $out"
Write-Host "port check $Target`: $portCheck"

# Password: file, else environment, else hidden prompt. Never printed.
$hadEnvPw = [bool]$env:GRAAL_PW
if ($PasswordFile) {
    $env:GRAAL_PW = (Get-Content -Raw $PasswordFile).TrimEnd("`r", "`n")
} elseif (-not $hadEnvPw) {
    $secure = Read-Host "Password for $Account" -AsSecureString
    $bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
    try { $env:GRAAL_PW = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($bstr) }
    finally { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr) }
}

function Find-Dumpcap {
    $cmd = Get-Command dumpcap.exe -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    foreach ($dir in @($env:ProgramFiles, ${env:ProgramFiles(x86)})) {
        if ($dir -and (Test-Path (Join-Path $dir 'Wireshark\dumpcap.exe'))) { return (Join-Path $dir 'Wireshark\dumpcap.exe') }
    }
    return $null
}

function Start-Capture {
    param([string]$OutBase, [int[]]$Ports, [string]$PortRange, [int]$TargetPort)
    $dumpcap = Find-Dumpcap
    if ($dumpcap) {
        $iface = $env:SEIHAI_CAPTURE_IF
        if (-not $iface) {
            $route = Get-NetRoute -DestinationPrefix 0.0.0.0/0 -ErrorAction SilentlyContinue |
                Sort-Object { $_.RouteMetric + $_.InterfaceMetric } | Select-Object -First 1
            if ($route) { $iface = $route.InterfaceAlias }
        }
        if (-not $iface) { Write-Warning 'no default-route interface; set SEIHAI_CAPTURE_IF'; return $null }
        $filter = "tcp and (portrange $PortRange or port $TargetPort)"
        $file = "$OutBase.pcap"
        $errLog = "$OutBase.dumpcap.txt"
        # -F pcap: classic pcap. dumpcap flushes after each batch, so stopping it
        # loses nothing already captured.
        $proc = Start-Process -FilePath $dumpcap -NoNewWindow -PassThru -RedirectStandardError $errLog `
            -ArgumentList @('-i', "`"$iface`"", '-f', "`"$filter`"", '-s', '0', '-F', 'pcap', '-q', '-w', "`"$file`"")
        Start-Sleep -Milliseconds 1500
        if ($proc.HasExited) { Write-Warning "dumpcap exited: $((Get-Content $errLog -ErrorAction SilentlyContinue) -join ' ')"; return $null }
        Write-Host "capturing with dumpcap on '$iface' to $file"
        return @{ Kind = 'dumpcap'; Proc = $proc; File = $file; ErrLog = $errLog }
    }
    $admin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if (-not $admin) { Write-Warning 'no Wireshark (dumpcap), and pktmon needs an elevated PowerShell; running without a capture'; return $null }
    # pktmon filters are machine-wide; leave anyone else's alone.
    if (((& pktmon filter list) -join "`n") -notmatch 'No filters') { Write-Warning 'pktmon already has filters; running without a capture'; return $null }
    foreach ($p in $Ports) { & pktmon filter add "seihai$p" -t TCP -p $p | Out-Null }
    $etl = "$OutBase.etl"
    & pktmon start --capture --pkt-size 0 --file-name $etl | Out-Null
    Write-Host "capturing with pktmon to $etl"
    return @{ Kind = 'pktmon'; Etl = $etl; File = "$OutBase.pcapng" }
}

function Stop-Capture {
    param($Capture)
    if (-not $Capture) { return }
    if ($Capture.Kind -eq 'dumpcap') {
        Start-Sleep -Milliseconds 500
        Stop-Process -Id $Capture.Proc.Id -ErrorAction SilentlyContinue
        $Capture.Proc.WaitForExit(5000) | Out-Null
        Remove-Item $Capture.ErrLog -ErrorAction SilentlyContinue
    } else {
        & pktmon stop | Out-Null
        & pktmon etl2pcap $Capture.Etl --out $Capture.File | Out-Null
        & pktmon filter remove | Out-Null
        Remove-Item $Capture.Etl -ErrorAction SilentlyContinue
    }
    if (Test-Path $Capture.File) { Write-Host "capture: $($Capture.File) ($((Get-Item $Capture.File).Length) bytes)" }
}

$debugRoot = Join-Path (Get-Location).Path '_debug'
$debugBefore = @(Get-ChildItem $debugRoot -Directory -Filter 'debug-*' -ErrorAction SilentlyContinue | ForEach-Object Name)
$cap = Start-Capture -OutBase (Join-Path $out 'capture') -Ports $Ports -PortRange $PortRange -TargetPort ([int]$port)

$log = New-Object IO.StreamWriter((Join-Path $out 'seihai_output.txt'), $false, (New-Object Text.UTF8Encoding $false))
$code = 1
try {
    $ErrorActionPreference = 'Continue'
    & $Seihai --online $Target --account $Account --password-env GRAAL_PW --version $Version @Extra 2>&1 |
        ForEach-Object { $line = "$_"; Write-Host $line; $log.WriteLine($line) }
    $code = $LASTEXITCODE
} finally {
    $log.Close()
    if (-not $hadEnvPw -or $PasswordFile) { Remove-Item Env:GRAAL_PW -ErrorAction SilentlyContinue }
    Write-Host 'client exited; stopping capture'
    Stop-Capture $cap
}

# Debug folders this run made (packets.log never holds the login line).
Get-ChildItem $debugRoot -Directory -Filter 'debug-*' -ErrorAction SilentlyContinue |
    Where-Object { $debugBefore -notcontains $_.Name } |
    ForEach-Object { Copy-Item $_.FullName (Join-Path $out $_.Name) -Recurse }

$zip = "$out.zip"
Compress-Archive -Path $out -DestinationPath $zip
& icacls $zip /inheritance:r /grant:r "$($env:USERNAME):F" | Out-Null
Write-Host ''
Write-Host "packed: $zip"
if ($cap -and $cap.Kind -eq 'dumpcap') { Write-Host "decode with: $Seihai capture show `"$($cap.File)`"" }
Write-Host 'The capture holds your login. Send the zip privately, not in a public channel.'
exit $code
