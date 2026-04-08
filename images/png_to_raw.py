from PIL import Image
from matplotlib import pyplot as plt
import subprocess

orig_file_path = "nonoti.png"
img = Image.open(orig_file_path).convert("RGBA")
pixels = img.load()

bg = Image.new("RGBA", img.size, (0, 0, 0, 255)) # Fill with the color you want for the transparent areas
bg.paste(img, (0, 0), img)


img.save(orig_file_path.replace(".png", "_transparent.png"))
bg.save(orig_file_path)

raw_file_path = orig_file_path.replace(".png", ".raw")
cmd = ["ffmpeg", "-i", orig_file_path, "-f", "rawvideo", "-pix_fmt", "rgb565be", raw_file_path]
subprocess.run(cmd, check=True)
