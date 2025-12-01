$ErrorActionPreference = "Stop"
$version = "0.1.4"

$outputDir = ".\release-builds\v$version"
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
}

$pubDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

$json = "{`n"
$json += "  `"version`": `"$version`",`n"
$json += "  `"notes`": `"Nouvelle version avec logo paresseux et correction du bug de la fenetre system tray`",`n"
$json += "  `"pub_date`": `"$pubDate`",`n"
$json += "  `"platforms`": {`n"
$json += "    `"windows-x86_64`": {`n"
$json += "      `"signature`": `"`",`n"
$json += "      `"url`": `"https://github.com/ay-bell/Flemme/releases/download/v$version/flemme-app-cuda-v$version-setup.msi`"`n"
$json += "    }`n"
$json += "  }`n"
$json += "}`n"

[System.IO.File]::WriteAllText("$outputDir\latest.json", $json, [System.Text.UTF8Encoding]::new($false))

Write-Host "Done: latest.json"
Get-Content "$outputDir\latest.json"
