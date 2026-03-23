
p8rs = p8rs or {}
do
	local stringify, stringify_table, proxy_metatable
	
	-- https://pico-8.fandom.com/wiki/Qsort
	local function qsort(a,c,l,r)
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
		elseif atype == "table" then return stringify_table(a) < stringify_table(b)
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
	
	local function stringify_memory(addr, count, cols)
		local bytes = ""
		for i = 0,count-1 do
			bytes = bytes .. sub(tostr(peek(addr + i), true), 5, 6)
			if cols and i % cols == cols - 1 then
				bytes = bytes .. " "
			end
		end
		return bytes
	end
	
	stringify_table = function(t)
		local out = ""
		local first = true

		for k=1,#t do
		local v = t[k]
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

	local function is_proxy(table)
		return type(table) == "table" and getmetatable(table) == proxy_metatable
	end
	
	stringify = function(v)
		if is_proxy(v) then return v.__label .. " " .. stringify_table(v.__inner) end
		local vtype = type(v)
		if vtype == "nil" then return "nil"
		elseif vtype == "string" then return escape(v)
		elseif vtype == "number" then return tostr(v, true)
		elseif vtype == "boolean" then return tostr(v)
		elseif vtype == "table" then return stringify_table(v)
		elseif vtype == "function" then return "[function]"
		elseif vtype == "thread" then return "[thread]"
		else assert(false, "Can't test for value of type " .. vtype)
		end
	end

	local function stringify_args(...)
		local argc = select("#", ...)
		local args = {...}
		local text = ""
		for i = 1,argc do
			if i > 1 then text ..= " | " end
			text ..= stringify(args[i])
		end
		return text
	end

	local function test_meta(...)
		printh("META | " .. stringify_args(...))
	end
	
	p8rs.test = function(name, ...)
		printh("TEST | " .. name .. " | " .. stringify_args(...))
	end
	
	p8rs.test_mem = function(name, addr, count)
		for chunk = addr,addr+count-1,32 do
			printh("MEM | " .. name .. " | " .. sub(tostr(chunk, true), 0, 6) .. " | " .. stringify_memory(chunk, min(addr-chunk+count, 32), 4))
		end
	end
	
	p8rs.test_scr = function(name)
		printh("SCR | " .. name .. " | pal | " .. stringify_memory(0x5f10, 16))
		for row = 0,127 do
			local leftpad
			if row < 10 then leftpad = "  "
			elseif row < 100 then leftpad = " "
			else leftpad = ""
			end
			printh("SCR | " .. name .. " | " .. leftpad .. row .. " | " .. stringify_memory(0x6000 + row * 64, 64))
		end
	end

	p8rs.test_err = function(name, callback)
		local co = cocreate(callback)
		local ok, message = coresume(co)
		assert(not ok, name .. " - Function did not throw an error")
		assert(type(message) == "string", name .. " - Error is not a string (got " .. type(message) .. ")")

		printh("ERR | " .. name .. " | " .. message)
	end

	proxy_metatable = {
		__index = function(table, key) test_meta("__index", table.__label, key); return table.__inner[key] end,
		__newindex = function(table, key, value) test_meta("__newindex", table.__label, key, value); table.__inner[key] = value end,
		__len = function(table) test_meta("__len", table.__label); return #table.__inner end,
		__eq = function(table1, table2) test_meta("__eq", table1.__label, table2.__label); return table1.__inner == table2.__inner end,
		__lt = function(value1, value2) test_meta("__lt", value1.__label, value2.__label); return table1.__inner < table2.__inner end,
		__le = function(value1, value2) test_meta("__le", value1.__label, value2.__label); return table1.__inner <= table2.__inner end,
		__add = function(value1, value2) test_meta("__add", value1.__label, value2.__label); return table1.__inner + table2.__inner end,
		__sub = function(value1, value2) test_meta("__sub", value1.__label, value2.__label); return table1.__inner - table2.__inner end,
		__mul = function(value1, value2) test_meta("__mul", value1.__label, value2.__label); return table1.__inner * table2.__inner end,
		__div = function(value1, value2) test_meta("__div", value1.__label, value2.__label); return table1.__inner / table2.__inner end,
		__idiv = function(value1, value2) test_meta("__idiv", value1.__label, value2.__label); return table1.__inner \ table2.__inner end,
		__mod = function(value1, value2) test_meta("__mod", value1.__label, value2.__label); return table1.__inner % table2.__inner end,
		__pow = function(value1, value2) test_meta("__pow", value1.__label, value2.__label); return table1.__inner ^ table2.__inner end,
		__and = function(value1, value2) test_meta("__and", value1.__label, value2.__label); return table1.__inner & table2.__inner end,
		__or = function(value1, value2) test_meta("__or", value1.__label, value2.__label); return table1.__inner | table2.__inner end,
		__xor = function(value1, value2) test_meta("__xor", value1.__label, value2.__label); return table1.__inner ^^ table2.__inner end,
		__shl = function(value1, value2) test_meta("__shl", value1.__label, value2.__label); return table1.__inner << table2.__inner end,
		__shr = function(value1, value2) test_meta("__shr", value1.__label, value2.__label); return table1.__inner >> table2.__inner end,
		__lshr = function(value1, value2) test_meta("__lshr", value1.__label, value2.__label); return table1.__inner >>> table2.__inner end,
		__rotl = function(value1, value2) test_meta("__rotl", value1.__label, value2.__label); return table1.__inner <<> table2.__inner end,
		__rotr = function(value1, value2) test_meta("__rotr", value1.__label, value2.__label); return table1.__inner >>< table2.__inner end,
		__concat = function(value1, value2) test_meta("__concat", value1.__label, value2.__label); return table1.__inner .. table2.__inner end,
		__unm = function(table) test_meta("__unm", table.__label); return -table.__inner end,
		__not = function(table) test_meta("__not", table.__label); return ~table.__inner end,
		__peek = function(table) test_meta("__peek", table.__label); return @table.__inner end,
		__peek2 = function(table) test_meta("__peek2", table.__label); return %table.__inner end,
		__peek4 = function(table) test_meta("__peek4", table.__label); return $table.__inner end,
		__call = function(table, ...) test_meta("__call", table.__label, ...); return table.__inner(...) end,
		__tostring = function(table) test_meta("__tostring", table.__label); return tostr(table.__inner) end,
		__pairs = function(table) test_meta("__pairs", table.__label); return pairs(table.__inner) end,
		__ipairs = function(table) test_meta("__ipairs", table.__label); return ipairs(table.__inner) end,
		__gc = function(table) test_meta("__gc", table.__label) end
	}
	proxy_metatable.__metatable = proxy_metatable;

	p8rs.test_proxy = function(label, inner)
		return setmetatable({
			__inner = inner or {},
			__label = label,
		}, proxy_metatable)
	end
	
	srand(0)
end
