printh("Filling")

for off = 0, 64 * 128 do
    poke(0x6000 + off, off % 256)
    if off % 256 == 255 then
        flip()
    end
end


printh("Done!")

function print_data()
    local str = ""
    for i = 0x5f00, 0x5f10 do
        str = str .. peek(i) .. " "
    end
    printh(str)
end

print_data()
pal({
    [1] = 10,
    [2] = 10,
    [5] = 10,
    [6] = 10,
}, 1)
print_data()
