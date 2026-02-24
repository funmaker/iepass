pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test("decimal", 123, -123)
p8rs.test("decimal fract", 123.456, -123.456)
p8rs.test("decimal overflow", 123456789.987654321, -123456789.987654321)
p8rs.test("decimal limit", 140737488355326.9, 140737488355327, 140737488355327.1, 140737488355327.9, 140737488355328, 140737488355328.1, 140737488355328.9, 140737488355329, 140737488355329.1)
p8rs.test("decimal limit fract", 140737488355327, 140737488355327.5, 140737488355327.25, 140737488355327.125, 140737488355327.0625, 140737488355327.03125, 140737488355327.015625, 140737488355327.0078125, 140737488355327.00390625, 140737488355327.001953125, 140737488355327.0009765625, 140737488355327.00048828125, 140737488355327.000244140625, 140737488355327.0001220703125, 140737488355327.00006103515625, 140737488355327.000030517578125, 140737488355327.0000152587890625)

p8rs.test("hex", 0xabc, -0xabc)
p8rs.test("hex fract", 0xabc.def, -0xabc.def)
p8rs.test("hex overflow", 0x123456789.abcdef123, -0x123456789.abcdef123)

p8rs.test("bin", 0b00100111, -0b100111)
p8rs.test("bin fract", 0b100111.100111, -0b100111.100111)
p8rs.test("bin overflow", 0b1100000001011010011.11100101010110000001, -0b1100000001011010011.11100101010110000001)

p8rs.test("boolean", true, false)
p8rs.test("nil", nil)

p8rs.test("unescaped control", "¹²³⁴⁵⁶⁷⁸ᵇᶜᵉᶠ")
p8rs.test("escaped control", "\0\*\#\-\|\+\^\a\b\t\n\v\f\r\14\15")
p8rs.test("ascii", " !\"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}")
p8rs.test("symbols", "▮■□⁙⁘‖◀▶「」¥•、。゛゜~○█▒🐱⬇️░✽●♥☉웃⌂⬅️😐♪🅾️◆…➡️★⧗⬆️ˇ∧❎▤▥◜◝")
p8rs.test("hiragana", "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんっゃゅょ")
p8rs.test("katakana", "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲンッャュョ")
p8rs.test("nested", "'")
p8rs.test("tednes", '"')
p8rs.test("unknown utf8", "𓂸")
p8rs.test("missing varsel", "⬇⬅🅾➡⬆")
p8rs.test("unexpected varsel", "abc️def️️️123 ️456")

p8rs.test("long", [[long string]])
p8rs.test("long multiline", [[long
long
looooong
string]])
p8rs.test("long unescaped control", [[¹²³⁴⁵⁶⁷⁸ᵇᶜᵉᶠ]])
p8rs.test("long escaped control", [[\0\*\#\-\|\+\^\a\b\t\n\v\f\r\14\15\"\']])
p8rs.test("long symbols", [[▮■□⁙⁘‖◀▶「」¥•、。゛゜~○█▒🐱⬇️░✽●♥☉웃⌂⬅️😐♪🅾️◆…➡️★⧗⬆️ˇ∧❎▤▥◜◝]])
p8rs.test("long unknown utf8", [[𓂸]])
p8rs.test("nested", [===[[[]=][][==[]====][===]==]===])
