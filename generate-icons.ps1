Add-Type -AssemblyName System.Drawing

$iconDir = "src-tauri\icons"
if (-not (Test-Path $iconDir)) {
    New-Item -ItemType Directory -Path $iconDir -Force | Out-Null
}

$sizes = @(32, 128, 256)
$names = @("32x32.png", "128x128.png", "128x128@2x.png")

for ($i = 0; $i -lt $sizes.Count; $i++) {
    $size = $sizes[$i]
    $name = $names[$i]
    
    $bmp = New-Object System.Drawing.Bitmap($size, $size)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    
    $orangeBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 152, 0))
    $g.Clear([System.Drawing.Color]::Transparent)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    
    $margin = [int]($size * 0.1)
    $rectSize = $size - (2 * $margin)
    $g.FillEllipse($orangeBrush, $margin, $margin, $rectSize, $rectSize)
    
    $fontSize = [int]($size * 0.4)
    $font = New-Object System.Drawing.Font("Arial", $fontSize, [System.Drawing.FontStyle]::Bold)
    $whiteBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
    
    $sf = New-Object System.Drawing.StringFormat
    $sf.Alignment = [System.Drawing.StringAlignment]::Center
    $sf.LineAlignment = [System.Drawing.StringAlignment]::Center
    
    $textRect = New-Object System.Drawing.RectangleF(0, 0, $size, $size)
    $g.DrawString("M", $font, $whiteBrush, $textRect, $sf)
    
    $bmp.Save("$iconDir\$name", [System.Drawing.Imaging.ImageFormat]::Png)
    
    $g.Dispose()
    $bmp.Dispose()
    
    Write-Host "Created: $name ($size x $size)"
}

Write-Host "`nIcons generated successfully!"
