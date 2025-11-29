pico-8 cartridge // http://www.pico-8.com
version 43

__lua__

local pokefn = { poke, poke2, poke4 }
local peekfn = { peek, peek2, peek4 }
local sizes = { 1, 2, 4 }

local bases = { 0x1234, 0x5f24, 0xfffd, 0x0000 }

local pattern = { 0xaaaa.aaaa, 0xbbbb.bbbb, 0xcccc.cccc, 0xdddd.dddd, 0xeeee.eeee, 0xffff.ffff }

for i_size = 1,3 do
        local poke = pokefn[i_size]
        local peek = peekfn[i_size]
        local size = sizes[i_size]
        p8rs.test("poke"..size.." exists", poke ~= nil)
        p8rs.test("peek"..size.." exists", peek ~= nil)
        if poke ~= nil and peek ~= nil then
                for i_base = 1,3 do
                        local base = bases[i_base]
                        local peeked = {}
                        for i_pattern=1,#pattern do
                                local val = pattern[i_pattern]
                                local addr = (i_pattern-1)*size + base
                                poke(addr, val)
                        end
                        for i_pattern=1,#pattern do
                                local val = pattern[i_pattern]
                                local addr = (i_pattern-1)*size + base
                                peeked[#peeked+1] = peek(addr)
                        end

                        local name = "Poke+Peek size="..size..", base="..sub(tostr(base, true), 3, 6)

                        printh('Running "'..name..' - singles"...')
                        p8rs.test(name, peeked)
                        local region_end = base + #pattern
                        if region_end < #pattern then
                                p8rs.test_mem(name .. " - singles, wrapped", 0, region_end + 1)
                                region_end = 0xffff
                        end
                        p8rs.test_mem(name .. " - singles", base, region_end - base)


                        printh('Running "'..name..' - table"...')
                        for i_pattern=1,#pattern do
                                local addr = (i_pattern-1)*size + base
                                poke(addr, 0)
                        end
                        poke(base, unpack(pattern))

                        if region_end < #pattern then
                                p8rs.test_mem(name .. " - table, wrapped", 0, region_end + 1)
                                region_end = 0xffff
                        end
                        p8rs.test_mem(name .. " - table", base, region_end - base)

                end
        end
end