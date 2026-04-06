pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
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

local entries = {}
for k, v in pairs(_ENV) do
	if not (type(k) == "number" and k == flr(k) and k > 0 and k <= #t) then
		add(entries, {k, v})
	end
end

qsort(entries, function(a, b) return a[1] < b[1] end)

for _, entry in ipairs(entries) do
	local dups = {}
	for _, other in ipairs(entries) do
		if entry[1] != other[1] and entry[2] == other[2] then
			add(dups, other[1])
		end
	end

	p8rs.test(entry[1], entry[2], unpack(dups))
end


