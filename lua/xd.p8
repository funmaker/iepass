pico-8 cartridge // http://www.pico-8.com
version 43
__lua__
printh("XD, hex display.")

function hex8(num)
    return sub(tostr(num, 3), 5, 6)
end

function hex16(num)
    return sub(tostr(num, 3), 3, 6)
end

local frame = 0
local base_addr = 0
function _update()
    if frame % 2 == 0 then
        local speed = (btn(4) or btn(4, 1)) and 0x200 or 0x20
        if btn(2) then
            base_addr = base_addr - speed
            if base_addr < 0 then base_addr = 0 end
        end
        if btn(3) then
            base_addr = base_addr + speed
            if base_addr < 0 then base_addr = 0x8000 - speed end
        end
    end

    frame = frame + 1
end

function _draw()
    cls()
    cursor(10, 10)

    for line = 0, 15 do
        local line_addr = base_addr + line * 8
        local text = hex16(line_addr) .. ": "

        for byte = 0, 7 do
            local val = peek(line_addr + byte)
            text = text .. hex8(val)
            if byte % 2 == 1 then text = text .. " " end
        end
        print(text)
    end
end
