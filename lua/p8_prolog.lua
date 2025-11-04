
█=█ or 0.5
▒=▒ or 23130.5
🐱=🐱 or 20767.5
⬇️=⬇️ or 3
░=░ or 32125.5
✽=✽ or -18402.5
●=● or -1632.5
♥=♥ or 20927.5
☉=☉ or -19008.5
웃=웃 or -26208.5
⌂=⌂ or -20192.5
⬅️=⬅️ or 0
😐=😐 or -24351.5
♪=♪ or -25792.5
🅾️=🅾️ or 4
◆=◆ or -20032.5
…=… or -2560.5
➡️=➡️ or 1
★=★ or -20128.5
⧗=⧗ or 6943.5
⬆️=⬆️ or 2
ˇ=ˇ or -2624.5
∧=∧ or 31455.5
❎=❎ or 5
▤=▤ or 3855.5
▥=▥ or 21845.5
;

printh("Prolog script entered. __TEST1="..tostr(__TEST1)..", __TEST2="..tostr(__TEST2))
__TEST2 = "prolog"

local args = { ... }

if #args ~= 1 or type(args[1]) ~= "function" then
    printh("[prolog] Did not get function with main code.")
    return
end

args[1]()

printh("Prolog script exited. __TEST1="..tostr(__TEST1)..", [set]__TEST2="..tostr(__TEST2))

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
