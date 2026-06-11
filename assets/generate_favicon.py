import zlib, struct

def make_png(width, height, pixels):
    raw_data = bytearray()
    for y in range(height):
        raw_data.append(0) # filter type 0
        for x in range(width):
            raw_data.extend(pixels[y * width + x])
    
    png = bytearray(b'\x89PNG\r\n\x1a\n')
    
    def make_chunk(tag, data):
        return struct.pack("!I", len(data)) + tag + data + struct.pack("!I", zlib.crc32(tag + data))
        
    png.extend(make_chunk(b'IHDR', struct.pack("!2I5B", width, height, 8, 2, 0, 0, 0)))
    png.extend(make_chunk(b'IDAT', zlib.compress(raw_data)))
    png.extend(make_chunk(b'IEND', b''))
    return png

# Colors
O = (255, 159, 28)  # Orange head
D = (229, 142, 26)  # Dark orange ears
S = (255, 191, 105) # Snout
B = (30, 32, 34)    # Black nose/eyes
R = (231, 29, 54)   # Red collar
G = (255, 215, 0)   # Gold tag
W = (30, 30, 30)    # Dark background

pixel_map = [
    W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W,
    W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W,
    W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W,
    W, W, W, D, W, W, W, W, W, W, W, D, W, W, W, W,
    W, W, D, D, W, W, W, W, W, W, D, D, W, W, W, W,
    W, W, D, O, O, O, O, O, O, O, O, D, W, W, W, W,
    W, W, O, O, O, O, O, O, O, O, O, O, W, W, W, W,
    W, W, O, O, B, O, O, O, O, B, O, O, W, W, W, W,
    W, W, O, O, O, O, O, O, O, O, O, O, W, W, W, W,
    W, W, O, O, O, S, S, S, S, O, O, O, W, W, W, W,
    W, W, O, O, S, B, B, S, S, O, O, O, W, W, W, W,
    W, W, O, O, S, S, S, S, S, O, O, O, W, W, W, W,
    W, W, W, O, O, O, O, O, O, O, O, W, W, W, W, W,
    W, W, W, W, R, R, R, R, R, R, W, W, W, W, W, W,
    W, W, W, W, W, G, G, W, W, W, W, W, W, W, W, W,
    W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W,
]

png_bytes = make_png(16, 16, pixel_map)
with open("c:/Users/charl/rusty/assets/favicon.png", "wb") as f:
    f.write(png_bytes)

print("favicon.png generated successfully.")
