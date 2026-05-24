$tagName = $args[0] -replace '^v', ''
if ($tagName -notmatch '^\d{8}$') {
    throw "Unsupported release tag '${tagName}'. Use YYYYMMDD, for example 20260524."
}

$year = [int]$tagName.Substring(2, 2)
$month = [int]$tagName.Substring(4, 2)
$day = [int]$tagName.Substring(6, 2)
$appVersion = "${year}.${month}.${day}"
$displayVersion = $tagName

$tauriConfPath = "src-tauri\tauri.conf.json"
$tauriConf = Get-Content -Path $tauriConfPath -Raw | ConvertFrom-Json
$tauriConf.version = $appVersion
$tauriConf | ConvertTo-Json -Depth 100 | Set-Content -Path $tauriConfPath

$env:TAURI_PRIVATE_KEY = Get-Content -Path "~\.tauri\launcherg-actions.key"
$env:TAURI_KEY_PASSWORD = Get-Content -Path "~\.tauri\launcherg-actions-pass.key"
$env:TAURI_SIGNING_PRIVATE_KEY = $env:TAURI_PRIVATE_KEY
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $env:TAURI_KEY_PASSWORD

Invoke-Expression "npm i"
Invoke-Expression "npm run tauri build"

$builtArtifactName = "Launcherg_${appVersion}_x64_ja-JP.msi.zip"
$releaseArtifactName = "Launcherg_${displayVersion}_x64_ja-JP.msi.zip"
$builtArtifactPath = ".\src-tauri\target\release\bundle\msi\${builtArtifactName}"
$builtSignaturePath = "${builtArtifactPath}.sig"
$releaseArtifactPath = ".\src-tauri\target\release\bundle\msi\${releaseArtifactName}"
$releaseSignaturePath = "${releaseArtifactPath}.sig"

if (!(Test-Path $builtArtifactPath)) {
    throw "Updater artifact was not found: ${builtArtifactPath}"
}
if (!(Test-Path $builtSignaturePath)) {
    throw "Updater signature was not found: ${builtSignaturePath}"
}
if ($builtArtifactPath -ne $releaseArtifactPath) {
    Copy-Item -Path $builtArtifactPath -Destination $releaseArtifactPath -Force
    Copy-Item -Path $builtSignaturePath -Destination $releaseSignaturePath -Force
}

$signature = (Get-Content -Path $releaseSignaturePath -Raw).Trim()
$pubDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
$updaterData = @{
    version = $appVersion
    notes = "See the release notes and demo before updating."
    pub_date = $pubDate
    signature = $signature
    url = "https://github.com/nnnSMM/Launcherg-Mod/releases/download/${tagName}/${releaseArtifactName}"
}
$updaterData | ConvertTo-Json -Depth 10 | Set-Content -Path ".tauri-updater.json"

$uiInfo = @{
    version = $appVersion
    displayVersion = $displayVersion
    pubDate = $pubDate
    demoUrl = "https://nnnsmm.github.io/Launcherg-Mod/"
    releaseUrl = "https://github.com/nnnSMM/Launcherg-Mod/releases/tag/${tagName}"
    highlights = @()
}
$uiInfo | ConvertTo-Json -Depth 10 | Set-Content -Path "update-info.json"

Invoke-Expression "npx -y prettier $tauriConfPath .tauri-updater.json update-info.json --write"

git add $tauriConfPath .tauri-updater.json update-info.json
git commit -m "Update for release $displayVersion"
git push origin main

git tag $tagName
git push origin $tagName
