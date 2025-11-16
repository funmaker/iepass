pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

local short_text = "short test"
local long_text = "testing print with a really long input text that will wrap twice"

function do_print(text)
  for offset=0,15 do
    cls()
    print("(initial) "..text, 12, offset)
    for i=1,11 do
      print("("..i..") "..text)
    end
  end
end

poke(0x5f36, 0x00)
do_print(short_text)
p8rs.test_scr("Flag not set ([0x5f36] = 0x00), short text")
do_print(long_text)
p8rs.test_scr("Flag not set ([0x5f36] = 0x00), long text")

poke(0x5f36, 0x40)
do_print(short_text)
p8rs.test_scr("Flag NO_PRINT_SCROLL ([0x5f36] = 0x40), short text")
do_print(long_text)
p8rs.test_scr("Flag NO_PRINT_SCROLL ([0x5f36] = 0x40), long text")

poke(0x5f36, 0x80)
do_print(short_text)
p8rs.test_scr("Flag PRINT_WRAP ([0x5f36] = 0x80), short text")
do_print(long_text)
p8rs.test_scr("Flag PRINT_WRAP ([0x5f36] = 0x80), long text")

poke(0x5f36, 0xc0)
do_print(short_text)
p8rs.test_scr("Flags NO_PRINT_SCROLL + PRINT_WRAP ([0x5f36] = 0xc0), short text")
do_print(long_text)
p8rs.test_scr("Flags NO_PRINT_SCROLL + PRINT_WRAP ([0x5f36] = 0xc0), long text")