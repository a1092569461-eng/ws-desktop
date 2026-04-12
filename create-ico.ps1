Add-Type -AssemblyName System.Drawing

$iconDir = "src-tauri\icons"
if (-not (Test-Path $iconDir)) {
    New-Item -ItemType Directory -Path $iconDir -Force | Out-Null
}

$sizes = @(16, 32, 48, 64, 128, 256)

$bmp256 = New-Object System.Drawing.Bitmap(256, 256)
$g = [System.Drawing.Graphics]::FromImage($bmp256)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias

$orangeBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 152, 0))
$g.Clear([System.Drawing.Color]::Transparent)
$g.FillEllipse($orangeBrush, 20, 20, 216, 216)

$font = New-Object System.Drawing.Font("Arial", 100, [System.Drawing.FontStyle]::Bold)
$whiteBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
$stringFormat = New-Object System.Drawing.StringFormat
$stringFormat.Alignment = [System.Drawing.StringAlignment]::Center
$stringFormat.LineAlignment = [System.Drawing.StringAlignment]::Center
$textRect = New-Object System.Drawing.RectangleF(0, 0, 256, 256)
$g.DrawString("M", $font, $whiteBrush, $textRect, $stringFormat)

$iconPath = "$iconDir\icon.ico"
$icon = [System.Drawing.Icon]::FromHandle($bmp256.GetHicon())
$fileStream = [System.IO.File]::Create($iconPath)
$icon.Save($fileStream)
$fileStream.Close()

Write-Host "Created: icon.ico"

$g.Dispose()
$bmp256.Dispose()

Write-Host "`nIcon generated successfully!"
