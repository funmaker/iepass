do
    local __menuitem = _menuitem
    local _load = load
    local _coresume = coresume
    local _extcmd = extcmd
    local _flipped = __flipped
	local _last_menuitem_index = 0
	local _last_menuitem_callback = function() end

	__flipped = nil
	_pausemenu = {}

	function menuitem(index_p, label, callback)
		index_p = index_p or _last_menuitem_index
		callback = callback or _last_menuitem_callback

		if (type(index_p) ~= "number") then
			stop("bad menuitem index")
		end

		local index = index_p & 0xff
		if (index < 1 or index > 5) then return end

		_pausemenu[index] = {}
		_pausemenu[index].label = label
		_pausemenu[index].callback = callback
		__menuitem(index_p, label)
	end

	function all(c)
		if c == nil or #c == 0 then
			return function() end
		end

		local i = 1
		local li = nil

		return function()
			if c[i] == li then i=i+1 end
			while c[i] == nil and i <= #c do i=i+1 end
			li = c[i]
			return li
		end
	end

	function foreach(c, _f)
		for i in all(c) do _f(i) end
	end

	function load(...)
		local res = _load(...)

		flip()
		if (_stat(107) > 0) then return true end
		if (_stat(107) == -1) then return false, "could not find cart" end
		if (_stat(107) == -2) then return false, "could not fetch cart" end
		if (_stat(107) == -3) then return false, "could not connect to bbs" end
		if (_stat(107) < 0 or res == nil) then return false, "access denied" end

		return res
	end

	function extcmd(cmd, ...)
		local res = _extcmd(cmd, ...)
		if cmd == "go_back" or cmd == "breadcrumb" then
			flip()
		end
		return res
	end

	function coresume(c,...)
		_flipped(false)
		local r0, r1 = _coresume(c, ...)

		while _flipped() and costatus(c) == "suspended" do
			_flipped(false)
			r0, r1 = _coresume(c, ...)
		end

        _flipped(false)
		return r0, r1
	end

	function flip()
		repeat
			local continue_menu = false
			for i=1,5 do
				local val = _get_menu_item_selected(i)
				if val then
					_last_menuitem_index = i
					_last_menuitem_callback = _pausemenu[i].callback
					continue_menu = _pausemenu[i].callback(val)
					if val & 3 > 0 then continue_menu = true end
				end
			end

			if continue_menu then
				extcmd("pause", 1)
				-- _superyield()
			end
		until not continue_menu

		__flip()
	end
end

local args = { ... }

assert(#args == 1 and type(args[1]) == "function", "[kernel] Expected function with main code.")

args[1]()

_end_of_program = 1

if _init ~= nil then _init() end

_set_mainloop_exists(0)

if _mainloop ~= nil then _set_mainloop_exists(1) end

if _update60 ~= nil then
    _set_fps(60)
    _update = nil
else
    _set_fps(30)
end

if _mainloop == nil and (_draw ~= nil or _update ~= nil or _update60 ~= nil) then
    _set_mainloop_exists(2)
    _mainloop = function()
        while true do
            _update_buttons(_update60 and 1 or 2)
            _startframe()

            if stat(7) == 60 then
                _mark_cpu(0)
                _update60()
            elseif stat(7) == 30 and _update60 then
                _update60()
                _update_buttons(1)
                _mark_cpu(0)
                _update60()
            elseif stat(7) == 30 and _update then
                _mark_cpu(0)
                _update()
            elseif stat(7) == 15 and _update then
                _update()
                _update_buttons(2)
                _mark_cpu(0)
                _update()
            end

            _mark_cpu(1)
            if _draw ~= nil then
                local di, res = 0, true
                while di < stat(11) and res do
                    res = _map_display(di)
                    if res then res = _draw() end
                    di = di + 1
                end
                _map_display(0)
            end

            _mark_cpu(2)
            _update_framerate()
           flip()
        end
    end
end

if _mainloop ~= nil then _mainloop() end
