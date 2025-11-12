pico-8 cartridge // http://www.pico-8.com
version 43

__lua__
cols = {-15, 1, -4, 12, 6, 7}
pal(cols, 1)

bayer = {1, 9, 3, 11, 13, 5, 15, 7, 4, 12, 2, 10, 16, 8, 14, 6}
pats = {}
for i = 0, 15 do
  val = 0
  for b = 0, 15 do
    if(bayer[b + 1] < i) val |= 1 << b
  end
  add(pats, val)
end

function log10(n)
	if (n <= 0) then return nil end
	local f, t = 0, 0
	while n < 0.5 do
		n *= 2.71828
		t -= 1
	end
	while n > 1.5 do
		n /= 2.71828
		t += 1
	end
	
	n -= 1
	for i = 9, 1, -1 do
	f = n*(1/i - f)
	end
	t += f
	return t / 2.30259
end

itermax = 100
for py = 0,127 do
	for px = 0,127 do
		x0 = (px / 128) * 2.2 - 1.7
		y0 = (py / 128) * 2.2 - 1.1
		x = 0.0
		y = 0.0
		escaped = 0
		for iter = 1,itermax do
			if x^2 + y^2 > 4 then
				escaped = iter
				break
			end
			x, y = x^2 - y^2 + x0, 2*x*y + y0
		end
		
		if escaped > 0 then
			col = (log10(escaped) / log10(itermax)) * (#cols + 2) - 2
			if (col < 0) col = 0
			fillp(pats[flr((col - flr(col)) * #bayer) + 1])
			printh(col - flr(col) .. " " .. flr((col - flr(col)) * #bayer) + 1 .. " " .. tostr(pats[flr((col - flr(col)) * #bayer) + 1], true))
			col = flr(col)
			
			pset(px, py, col | ((col + 1) << 4))
		else
			pset(px, py, 0)
		end
	end
	flip()
end

function _draw() end
function _update() end
