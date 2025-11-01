
p8rs = p8rs or {}
do
	local function escape(str)
		local out = ""
		for i = 1,#str do
			local ch = chr(str, i)
			if ch == 0 then out = out .. "\\0"
			elseif ch == 9 then out = out .. "\\t"
			elseif ch == 10 then out = out .. "\\n"
			elseif ch == 13 then out = out .. "\\r"
			elseif ch == 34 then out = out .. "\\\""
			elseif ch == 92 then out = out .. "\\\\"
			else out = out .. sub(str, i, i)
			end
		end
		return '"' .. out .. '"'
	end
	
	local function stringifyMemory(addr, count, cols)
		local bytes = ""
		for i = 0,count-1 do
			bytes = bytes .. sub(tostr(peek(addr + i), true), 5, 6)
			if cols and i % cols == cols - 1 then
				bytes = bytes .. " "
			end
		end
		return bytes
	end
	
	local stringify
	local function stringifyTable(t)
		local out = ""
		local first = true
		local seq = 0
		
		for k, v in pairs(t) do
			if first then first = false
			else out = out .. ", "
			end
			
			if type(k) == "number" and type(seq) == "number" and k == seq + 1 then
				seq = k
			else
				seq = nil
				out = out .. "[" .. stringify(k) .. "] = "
			end
			
			out = out .. stringify(v)
		end
		return '{ ' .. out .. ' }'
	end
	
	stringify = function(v)
		local vtype = type(v)
		if vtype == "nil" then return "nil"
		elseif vtype == "string" then return escape(v)
		elseif vtype == "number" then return tostr(v, true)
		elseif vtype == "boolean" then return tostr(v)
		elseif vtype == "table" then return stringifyTable(v)
		else assert(false, "Can't test for value of type " .. vtype)
		end
	end
	
	p8rs.test = function(name, value)
		printh("TEST | " .. name .. " | " .. stringify(value))
	end
	
	p8rs.test_mem = function(name, addr, count)
		for chunk = addr,addr+count-1,32 do
			printh("MEM | " .. name .. " | " .. sub(tostr(chunk, true), 0, 6) .. " | " .. stringifyMemory(chunk, min(addr-chunk+count, 32), 4))
		end
	end
	
	p8rs.test_scr = function(name)
		local row = 0
		printh("SRC | " .. name .. " | pal | " .. stringifyMemory(0x5f10, 16))
		for chunk = 0x6000,0x7fff,64 do
			local leftpad
			if row < 10 then leftpad = "  "
			elseif row < 100 then leftpad = " "
			else leftpad = ""
			end
			printh("SRC | " .. name .. " | " .. leftpad .. row .. " | " .. stringifyMemory(chunk, 64))
			row = row + 1
		end
	end
end
