pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
ovalfill(56, 56, 72, 72)
ovalfill(16, 64, 32, 32, 8)
ovalfill(104, 104, 48, 88)
p8rs.test_scr("basic");

cls()
ovalfill(56, 56, 72, 72)
ovalfill(16.9, 64.9, 32.9, 32.9, 8)
ovalfill(104.9, 104.9, 48.9, 88.9)
p8rs.test_scr("frac");

cls()
ovalfill(56, -56, 72, 72)
ovalfill(-16.9, 64.9, 32.9, 32.9, 8)
ovalfill(104.9, 104.9, -48.9, 88.9)
p8rs.test_scr("outside");

cls()
color(11)
ovalfill(56, 56, 72, 72)
ovalfill(16, 64, 32, 32, 8)
ovalfill(104, 104, 48, 88)
p8rs.test_scr("color");

cls()
color(8)
ovalfill(56, 56, 72)
color(11)
ovalfill(16, 64)
color(12)
ovalfill(104)
color(14)
ovalfill()
p8rs.test_scr("less args");

cls()
camera(-63, -65)
ovalfill(-8, -8, 8, 8)
ovalfill(-48, 0, -32, -32, 8)
ovalfill(40, 40, -16, 16)
camera()
p8rs.test_scr("camera");

cls()
clip(26, 34, 68, 60)
camera(-63, -65)
ovalfill(-8, -8, 8, 8)
ovalfill(-48, 0, -32, -32, 8)
ovalfill(40, 40, -16, 16)
camera()
clip()
p8rs.test_scr("clip");

cls()
clip(26, 34, 68, 60)
camera(-63, -65)
fillp(✽)
ovalfill(-8, -8, 8, 8)
fillp(0b0011001111001100)
ovalfill(-48, 0, -32, -32, 0xb8)
fillp(♥)
ovalfill(40, 40, -16, 16)
fillp(…)
ovalfill(-32, -32, 32, 32, 0x0c)
camera()
fillp()
clip()
p8rs.test_scr("fill pattern");

for e = -2,2,0.1 do
  cls()
  s = 2^e
  for i = 64,0,-1 do
    ovalfill(64 - i * 4 * s, 64 - i * 4 / s, 64 + i * 4 * s, 64 + i * 4 / s, i % 15 + 1)
  end
  p8rs.test_scr("concentric " .. s);
end

for e = -2,2,0.1 do
  cls()
  s = 2^e
  for i = 64,0,-1 do
    ovalfill(64 - i * 4 * s, 64 - i * 4 / s, 65 + i * 4 * s, 65 + i * 4 / s, i % 15 + 1)
  end
  p8rs.test_scr("concentric offcenter " .. s);
end