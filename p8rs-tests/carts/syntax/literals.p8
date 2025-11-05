pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test("decimal", 123)
p8rs.test("decimal neg", -123)
p8rs.test("decimal fract", 123.456)
p8rs.test("decimal fract neg", -123.456)
p8rs.test("decimal overflow", 123456789.987654321)
p8rs.test("decimal overflow neg", -123456789.987654321)

p8rs.test("hex", 0xabc)
p8rs.test("hex neg", -0xabc)
p8rs.test("hex fract", 0xabc.def)
p8rs.test("hex fract neg", -0xabc.def)
p8rs.test("hex overflow", 0x123456789.abcdef123)
p8rs.test("hex overflow neg", -0x123456789.abcdef123)

p8rs.test("bin", 0b00100111)
p8rs.test("bin neg", -0b100111)
p8rs.test("bin fract", 0b100111.100111)
p8rs.test("bin fract neg", -0b100111.100111)
p8rs.test("bin overflow", 0b1100000001011010011.11100101010110000001)
p8rs.test("bin overflow neg", -0b1100000001011010011.11100101010110000001)

p8rs.test("true", true)
p8rs.test("false", false)
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
