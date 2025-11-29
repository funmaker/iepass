█=█ 	or 0x0000.8000
▒=▒ 	or 0x5a5a.8000
🐱=🐱 	or 0x511f.8000
░=░ 	or 0x7d7d.8000
✽=✽ 	or 0xb81d.8000
●=● 	or 0xf99f.8000
♥=♥ 	or 0x51bf.8000
☉=☉ 	or 0xb5bf.8000
웃=웃 	or 0x999f.8000
⌂=⌂ 	or 0xb11f.8000
😐=😐 	or 0xa0e0.8000
♪=♪ 	or 0x9b3f.8000
◆=◆ 	or 0xb1bf.8000
…=… 	or 0xf5ff.8000
★=★ 	or 0xb15f.8000
⧗=⧗ 	or 0x1b1f.8000
ˇ=ˇ 	or 0xf5bf.8000
∧=∧ 	or 0x7adf.8000
▤=▤ 	or 0x0f0f.8000
▥=▥ 	or 0x5555.8000
⬅️=⬅️ or 0
➡️=➡️ or 1
⬆️=⬆️ or 2
⬇️=⬇️ or 3
🅾️=🅾️ or 4
❎=❎ or 5


local args = { ... }

assert(#args == 1 and type(args[1]) == "function", "[kernel] Did not get function with main code.")

args[1]()

_end_of_program = 1

if _init ~= nil then _init() end

if _update60 ~= nil then
    _set_fps(60)
    _update = nil
else
    _set_fps(30)
end

if _mainloop == nil and (_draw ~= nil or _update ~= nil or _update60 ~= nil) then
    _mainloop = function()
        while true do
            local fps = stat(7)
            local update = _update or _update60
            local update_is_60 = update == _update60

            if fps > 45 then
                update()
            elseif fps > 20 then
                update()
                if update_is_60 then update() end
            else
                update()
                update()
            end

            if (_draw ~= nil) then
                _draw()
            end

            flip()
        end
    end
end

if _mainloop ~= nil then _mainloop() end
