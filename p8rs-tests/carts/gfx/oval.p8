pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
oval(56, 56, 72, 72)
oval(16, 64, 32, 32, 8)
oval(104, 104, 48, 88)
p8rs.test_scr("basic");

cls()
oval(56, 56, 72, 72)
oval(16.9, 64.9, 32.9, 32.9, 8)
oval(104.9, 104.9, 48.9, 88.9)
p8rs.test_scr("frac");

cls()
oval(56, -56, 72, 72)
oval(-16.9, 64.9, 32.9, 32.9, 8)
oval(104.9, 104.9, -48.9, 88.9)
p8rs.test_scr("outside");

cls()
color(11)
oval(56, 56, 72, 72)
oval(16, 64, 32, 32, 8)
oval(104, 104, 48, 88)
p8rs.test_scr("color");

cls()
color(8)
oval(56, 56, 72)
color(11)
oval(16, 64)
color(12)
oval(104)
color(14)
oval()
p8rs.test_scr("less args");

cls()
camera(-63, -65)
oval(-8, -8, 8, 8)
oval(-48, 0, -32, -32, 8)
oval(40, 40, -16, 16)
camera()
p8rs.test_scr("camera");

cls()
clip(26, 34, 68, 60)
camera(-63, -65)
oval(-8, -8, 8, 8)
oval(-48, 0, -32, -32, 8)
oval(40, 40, -16, 16)
camera()
clip()
p8rs.test_scr("clip");

cls()
clip(26, 34, 68, 60)
camera(-63, -65)
fillp(✽)
oval(-8, -8, 8, 8)
fillp(0b0011001111001100)
oval(-48, 0, -32, -32, 0xb8)
fillp(♥)
oval(40, 40, -16, 16)
fillp(…)
oval(-32, -32, 32, 32, 0x0c)
camera()
fillp()
clip()
p8rs.test_scr("fill pattern");

for e = -2,2,0.1 do
  cls()
  s = 2^e
  for i = 64,0,-1 do
    oval(64 - i * 4 * s, 64 - i * 4 / s, 64 + i * 4 * s, 64 + i * 4 / s, i % 15 + 1)
  end
  p8rs.test_scr("concentric " .. s);
end

for e = -2,2,0.1 do
  cls()
  s = 2^e
  for i = 64,0,-1 do
    oval(64 - i * 4 * s, 64 - i * 4 / s, 65 + i * 4 * s, 65 + i * 4 / s, i % 15 + 1)
  end
  p8rs.test_scr("concentric offcenter " .. s);
end
