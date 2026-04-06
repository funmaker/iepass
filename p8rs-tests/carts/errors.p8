pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
p8rs.test_err("assert", function() assert(false, "custom message") end)
p8rs.test_err("c error", function() __type() end)
p8rs.test_err("lua error", function() (nil)[123] = 5 end)
