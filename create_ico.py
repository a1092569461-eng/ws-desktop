from PIL import Image
import os

icon_dir = "src-tauri/icons"
png_path = os.path.join(icon_dir, "32x32.png")
ico_path = os.path.join(icon_dir, "icon.ico")

if os.path.exists(png_path):
    img = Image.open(png_path)
    img.save(ico_path, format='ICO', sizes=[(16, 16), (32, 32), (48, 48), (64, 64)])
    print(f"Created {ico_path} from {png_path}")
else:
    print(f"PNG not found: {png_path}")
    img = Image.new('RGBA', (32, 32), (0, 0, 0, 0))
    from PIL import ImageDraw
    draw = ImageDraw.Draw(img)
    draw.ellipse([2, 2, 30, 30], fill=(255, 152, 0, 255))
    img.save(ico_path, format='ICO', sizes=[(16, 16), (32, 32), (48, 48), (64, 64)])
    print(f"Created {ico_path}")

print("Done!")
