pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
fillp(0b0011001111001100)
p8rs.test_mem("Basic", 0x5f00, 0x80)

fillp(0b1100111100110011.1)
p8rs.test_mem("Transparent", 0x5f00, 0x80)

fillp(0b0011001111001100.1111111111111111)
p8rs.test_mem("Extra flags", 0x5f00, 0x80)

pats = {█, ▒, 🐱, ⬇️, ░, ✽, ●, ♥, ☉, 웃, ⌂, ⬅️, 😐, ♪, 🅾️, ◆, …, ➡️, ★, ⧗, ⬆️, ˇ, ∧, ❎, ▤, ▥}
for k, pat in ipairs(pats) do
	x = k % 6
	y = k \ 6
	fillp(pat)
	rectfill(x * 21, y * 21, x * 21 + 19, y * 21 + 19, 8 + (k % 7))
end
p8rs.test_mem("Built in", 0x5f00, 0x80)



