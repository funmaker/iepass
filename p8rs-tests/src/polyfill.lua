
p8rs = p8rs or {}
do
	local stringify, stringifyTable
	
	-- https://pico-8.fandom.com/wiki/Qsort
	function qsort(a,c,l,r)
		c,l,r=c or function(a,b) return a<b end,l or 1,r or #a
		if l<r then
			if c(a[r],a[l]) then
				a[l],a[r]=a[r],a[l]
			end
			local lp,k,rp,p,q=l+1,l+1,r-1,a[l],a[r]
			while k<=rp do
				local swaplp=c(a[k],p)
				-- "if a or b then else"
				-- saves a token versus
				-- "if not (a or b) then"
				if swaplp or c(a[k],q) then
				else
					while c(q,a[rp]) and k<rp do
						rp=rp-1
					end
					a[k],a[rp],swaplp=a[rp],a[k],c(a[rp],p)
					rp=rp-1
				end
				if swaplp then
					a[k],a[lp]=a[lp],a[k]
					lp=lp+1
				end
				k=k+1
			end
			lp=lp-1
		rp=rp+1
		-- sometimes lp==rp, so 
		-- these two lines *must*
		-- occur in sequence;
		-- don't combine them to
		-- save a token!
		a[l],a[lp]=a[lp],a[l]
		a[r],a[rp]=a[rp],a[r]
		qsort(a,c,l,lp-1       )
		qsort(a,c,  lp+1,rp-1  )
		qsort(a,c,       rp+1,r)
		end
	end
	
	local function compareAny(a, b)
		local atype = type(a)
		local btype = type(b)
		if atype ~= btype then return atype < btype
		elseif atype == "nil" then return false
		elseif atype == "string" then return a < b
		elseif atype == "number" then return a < b
		elseif atype == "boolean" then return (a and 1 or 0) < (b and 1 or 0)
		elseif atype == "table" then return stringifyTable(a) < stringifyTable(b)
		else assert(false, "Can't compare values of type " .. atype)
		end
	end
	
	local function escape(str)
		local out = ""
		for i = 1,#str do
			local ch = ord(str, i)
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
	
	stringifyTable = function(t)
		local out = ""
		local first = true
		
		for _, v in ipairs(t) do
			if first then first = false
			else out = out .. ", "
			end
			out = out .. stringify(v)
		end
		
		local entries = {}
		for k, v in pairs(t) do
			if not (type(k) == "number" and k == flr(k) and k > 0 and k <= #t) then
				add(entries, {k, v})
			end
		end
		
		qsort(entries, function(a, b) return compareAny(a[1], b[1]) end)
		
		for _, entry in ipairs(entries) do
			if first then first = false
			else out = out .. ", "
			end
			
			out = out .. "[" .. stringify(entry[1]) .. "] = " .. stringify(entry[2])
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
		printh("SCR | " .. name .. " | pal | " .. stringifyMemory(0x5f10, 16))
		for row = 0,127 do
			local leftpad
			if row < 10 then leftpad = "  "
			elseif row < 100 then leftpad = " "
			else leftpad = ""
			end
			printh("SCR | " .. name .. " | " .. leftpad .. row .. " | " .. stringifyMemory(0x6000 + row * 64, 64))
		end
	end
end
