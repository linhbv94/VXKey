"""Rasterize the four user-supplied polygon SVGs for native tray RGBA.

Deliberately rejects other drawing primitives instead of silently losing shapes.
No external image library, fonts, downloads, or generated replacement glyphs.
"""
from pathlib import Path
import hashlib
import json
import xml.etree.ElementTree as ET

root = Path(__file__).resolve().parents[1]
out = root / 'desktop/src-tauri/assets'
out.mkdir(parents=True, exist_ok=True)
manifest = {}
size, samples = 36, 4

def inside(x, y, polygon):
    odd = False
    previous = polygon[-1]
    for current in polygon:
        ax, ay = previous; bx, by = current
        if (ay > y) != (by > y) and x < (bx - ax) * (y - ay) / (by - ay) + ax:
            odd = not odd
        previous = current
    return odd

for style in ('serious', 'playful'):
    for mode in ('vi', 'en'):
        name = f'menubar_{style}_{mode}'
        source = root / f'{name}.svg'
        svg = ET.fromstring(source.read_bytes())
        assert svg.attrib['viewBox'] == '0 0 64 64'
        polygons = []
        for element in svg.iter():
            tag = element.tag.split('}')[-1]
            assert tag in {'svg','defs','style','polygon'}, tag
            if tag == 'polygon':
                values = list(map(float, element.attrib['points'].replace(',', ' ').split()))
                assert len(values) % 2 == 0
                polygons.append(list(zip(values[::2], values[1::2])))
        assert polygons
        rgba = bytearray()
        for y in range(size):
            for x in range(size):
                coverage = sum(any(inside((x+(sx+.5)/samples)*64/size, (y+(sy+.5)/samples)*64/size, p) for p in polygons) for sy in range(samples) for sx in range(samples))
                rgba.extend((255,255,255,round(255*coverage/(samples*samples))))
        target = out / f'{name}.rgba'
        target.write_bytes(rgba)
        manifest[name] = {'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(), 'rgba_sha256':hashlib.sha256(rgba).hexdigest(), 'width':size, 'height':size}
(out/'tray_asset_manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('Native tray assets: 4 SVG polygons → 36x36 RGBA')
