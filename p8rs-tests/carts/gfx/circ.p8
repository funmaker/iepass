pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
circ(32, 96)
circ(96, 96, 16, 8)
circ(64, 64, 16)
p8rs.test_scr("basic");

cls()
color(11)
circ(32, 96)
circ(96, 96, 16, 8)
circ(64, 64, 16)
p8rs.test_scr("color");

cls()
color(8)
circ(32)
color(11)
circ()
p8rs.test_scr("less args");

cls()
for _, x in ipairs({ 10, 42.2, 74.5, 106.7 }) do
	for k, r in ipairs({ 8, 8.2, 8.5, 8.7, 9, 9.2, 9.5, 9.7 }) do
		circ(x + (k % 2) * 16, k * 16 - 6, r, 6)
	end
end
p8rs.test_scr("frac");

cls()
flags = peek(0x5f36)
poke(0x5f36, flags | 0x2)
for _, x in ipairs({ 10, 42.2, 74.5, 106.7 }) do
	for k, r in ipairs({ 8, 8.4, 8.5, 8.9, 9, 9.1, 9.5, 9.6 }) do
		circ(x + (k % 2) * 16, k * 16 - 6, r, 6)
	end
end
poke(0x5f36, flags)
p8rs.test_scr("frac 0x5f36 flag");

cls()
camera(-63, -65)
circ(-32, 16)
circ(32, 16, 16, 8)
circ(0, 0, 16)
camera()
p8rs.test_scr("camera");

cls()
camera(-63, -65)
clip(30, 34, 64, 50)
circ(-32, 16)
circ(32, 16, 16, 8)
circ(0, 0, 16)
camera()
clip()
p8rs.test_scr("clip");

cls()
camera(-63, -65)
clip(30, 34, 64, 50)
fillp(✽)
circ(-32, 16)
fillp(0b0011001111001100)
circ(32, 16, 16, 0xb8)
fillp(♥)
circ(0, 0, 16)
fillp(…)
circ(0, 0, 32, 0x0c)
camera()
fillp()
clip()
p8rs.test_scr("fill pattern");

for s = 0,3.5,0.5 do
  cls()
  for i = 24,0,-1 do
    circ(64, 64, i * 4 + s, i % 15 + 1)
  end
  p8rs.test_scr("concentric " .. s);
end

flags = peek(0x5f36)
poke(0x5f36, flags | 0x2)
for s = 0,3.5,0.5 do
  cls()
  for i = 24,0,-1 do
    circ(64, 64, i * 4 + s, i % 15 + 1)
  end
  p8rs.test_scr("concentric 0x5f36 " .. s);
end
poke(0x5f36, flags)

