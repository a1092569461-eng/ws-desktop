Add-Type -AssemblyName System.Drawing

$iconDir = "src-tauri\icons"
$pngPath = "$iconDir\32x32.png"
$icoPath = "$iconDir\icon.ico"

if (Test-Path $pngPath) {
    $bmp = [System.Drawing.Bitmap]::FromFile((Resolve-Path $pngPath))
    $icon = [System.Drawing.Icon]::FromHandle($bmp.GetHicon())
    
    $stream = [System.IO.File]::Create($icoPath)
    $icon.Save($stream)
    $stream.Close()
    $bmp.Dispose()
    
    Write-Host "Created icon.ico from 32x32.png"
} else {
    Write-Host "Creating new icon..."
    
    $bmp = New-Object System.Drawing.Bitmap(32, 32)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    
    $orangeBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 152, 0))
    $g.Clear([System.Drawing.Color]::Transparent)
    $g.FillEllipse($orangeBrush, 2, 2, 28, 28)
    
    $font = New-Object System.Drawing.Font("Arial", 14, [System.Drawing.FontStyle]::Bold)
    $whiteBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
    $stringFormat = New-Object System.Drawing.StringFormat
    $stringFormat.Alignment = [System.Drawing.StringAlignment]::Center
    $stringFormat.LineAlignment = [System.Drawing.StringAlignment]::Center
    $textRect = New-Object System.Drawing.RectangleF(0, 0, 32, 32)
    $g.DrawString("M", $font, $whiteBrush, $textRect, $stringFormat)
    
    $icon = [System.Drawing.Icon]::FromHandle($bmp.GetHicon())
    $stream = [System.IO.File]::Create($icoPath)
    $icon.Save($stream)
    $stream.Close()
    
    $g.Dispose()
    $bmp.Dispose()
    
    Write-Host "Created icon.ico"
}

Write-Host "Done!"
