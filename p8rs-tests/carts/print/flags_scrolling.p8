pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

local short_text = "short test"
local long_text = "testing print with a really long input text that will wrap twice"
local offset = 0
function do_print(text)
  offset = (offset + 1) % 15
  cls()
  print("(initial) "..text, 12, 90 + offset)
  for i=1,3 do
    print("("..i..") "..text)
  end
end

function do_test(name)
  poke(0x5f36, 0x00)
  do_print(short_text)
  p8rs.test_scr("[0x5f36] = 0x00, short text, "..name)
  do_print(long_text)
  p8rs.test_scr("[0x5f36] = 0x00, long text, "..name)

  poke(0x5f36, 0x40)
  do_print(short_text)
  p8rs.test_scr("[0x5f36] = 0x40, short text, "..name)
  do_print(long_text)
  p8rs.test_scr("[0x5f36] = 0x40, long text, "..name)

  poke(0x5f36, 0x80)
  do_print(short_text)
  p8rs.test_scr("[0x5f36] = 0x80, short text, "..name)
  do_print(long_text)
  p8rs.test_scr("[0x5f36] = 0x80, long text, "..name)

  poke(0x5f36, 0xc0)
  do_print(short_text)
  p8rs.test_scr("[0x5f36] = 0xc0, short text, "..name)
  do_print(long_text)
  p8rs.test_scr("[0x5f36] = 0xc0, long text, "..name)
end

do_test("plain")

local flags = { "WIDE", "TALL" }

for k, v in ipairs(flags) do
  local val = 1 | (1 << k)
  poke(0x5f58, val)
  do_test("[0x5f58] = 0x" .. sub(tostr(val, 1), 5, 6) .. " (" .. v .. ")")
end
