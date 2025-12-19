pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

local short_text = "short test"
local long_text = "testing print with a really long input text that will wrap twice"
function do_print(text)
  cls(2)
  print("(initial) "..text, 12, 20)
  for i=1,2 do
    print("("..i..") "..text)
  end
end

function do_test(name)
  do_print(short_text)
  p8rs.test_scr("[0x5f36] = 0x00, short text, "..name)
  do_print(long_text)
  p8rs.test_scr("[0x5f36] = 0x00, long text, "..name)
end

do_test("plain")

local flags = { "PADDING", "WIDE", "TALL", "SOLID_BG", "INVERT", "DOTTY", "CUSTOM_FONT" }
local tested = {
        { 1 | 1<<2 | 1<<3 | 0<<5 | 1<<6, "Dotty" },
        { 1 | 1<<2 | 1<<3 | 1<<5 | 1<<6, "Dotty Invert" },
        { 1 | 1<<2 | 0<<3 | 0<<5 | 1<<6, "Dotty Wide" },
        { 1 | 0<<2 | 1<<3 | 0<<5 | 1<<6, "Dotty Tall" },
}

for k, v in ipairs(flags) do
  local val = 1 | (1 << k)
  tested[#tested+1] = { val, v }
end

for k, v in ipairs(tested) do
  local val = v[1]
  local name = v[2]
  poke(0x5f58, val)
  do_test("[0x5f58] = 0x" .. sub(tostr(val, 1), 5, 6) .. " (" .. name .. ")")
end
