from PIL import Image

# Chemin de l'image source (à adapter si besoin)
input_path = r"C:/Users/nathan/Documents/Scripts/BackupGPT/trusttemp/standby.png"
output_path = "script/trusttemp/standby.ppm"

img = Image.open(input_path).convert("RGB")
img.save(output_path, format="PPM")
print(f"Image convertie : {output_path}")
