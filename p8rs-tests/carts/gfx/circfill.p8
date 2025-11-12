pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
circfill(32, 96)
circfill(96, 96, 16, 8)
circfill(64, 64, 16)
p8rs.test_scr("Basic");

cls()
color(11)
circfill(32, 96)
circfill(96, 96, 16, 8)
circfill(64, 64, 16)
p8rs.test_scr("Color");

cls()
color(8)
circfill(32)
color(11)
circfill()
p8rs.test_scr("Less args");

cls()
for _, x in ipairs({16, 48.2, 80.5, 112.7}) do
	for k, r in ipairs({ 8, 8.2, 8.5, 8.7, 9, 9.2, 9.5, 9.7 }) do
		circfill(x + (k % 2) * 16, k * 16, r)
	end
end
p8rs.test_scr("Frac");

cls()
flags = peek(0x5f36)
poke(0x5f36, flags | 0x2)
for _, x in ipairs({16, 48.2, 80.5, 112.7}) do
	for k, r in ipairs({ 8, 8.2, 8.5, 8.7, 9, 9.2, 9.5, 9.7 }) do
		circfill(x + (k % 2) * 16, k * 16, r)
	end
end
poke(0x5f36, flags)
p8rs.test_scr("Frac 0x5f36 flag");

cls()
camera(-63, -65)
circfill(-32, 16)
circfill(32, 16, 16, 8)
circfill(0, 0, 16)
p8rs.test_scr("Camera");

cls()
camera(-63, -65)
clip(30, 34, 64, 50)
circfill(-32, 16)
circfill(32, 16, 16, 8)
circfill(0, 0, 16)
p8rs.test_scr("Clip");

cls()
camera(-63, -65)
clip(30, 34, 64, 50)
fillp(✽)
circfill(-32, 16)
fillp(0b0011001111001100)
circfill(32, 16, 16, 0xb8)
fillp(♥)
circfill(0, 0, 16)
fillp(…)
circfill(0, 0, 32, 0x0c)
p8rs.test_scr("Fill Pattern");
