pico-8 cartridge // http://www.pico-8.com
version 43
__lua__

pad = 0
dpl = false
balx = 0
baly = 0
balvx = 0
balvy = 0
points = 0
brics = {}

function _init()
	for x=0,7 do
		for y=0,7 do
			add(brics, { x=x*16 + 2, y=y*4+10, c=y+8, l=true, p=(8-y)*100 })
		end
	end
end

function _update()
	pad += (btn(⬅️) and -2 or 0) + (btn(➡️) and 2 or 0)
	pad = mid(0, pad, 112)
	if dpl then
		balx += balvx
		baly += balvy
		if balx <= 0 or balx >= 127 then
			balvx = -balvx
			balx = mid(0, balx, 127)
		end
		if baly <= 0 then
			balvy = -balvy
			baly = mid(0, baly, 127)
		end
		if baly > 127 then
			dpl = false
		end
		if baly >= 108 and balx >= pad and balx <= pad + 15 then
			ang = (balx - pad - 8) / 26
			baly = 109
			balvy = -abs(cos(ang) * 3)
			balvx = sin(ang) * -3
		end
		for k, v in ipairs(brics) do
			if v.l and balx >= v.x and balx <= v.x + 14 and baly >= v.y and baly <= v.y + 4 then
				if balx <= v.x + 2 or balx >= v.x + 12 then
				 balvx = -balvx
				else
					balvy = -balvy
				end
				v.l = false
				points += v.p
			end
		end
	else
		balx = pad + 7
		baly = 108
		if btnp(🅾️) then
			dpl = true
			balvx = 0
			balvy = -3
		end
	end
end

function _draw()
	cls()
	color(7)
	rectfill(pad, 110, pad+15, 112)
	rectfill(balx, baly, balx+1, baly+1)
	for _,v in ipairs(brics) do
		if v.l then
			rectfill(v.x, v.y, v.x + 12, v.y + 1, v.c)
		end
	end
	print("★" .. points, 2, 2, 6)
	print("♥♥♥", 105, 2, 8)
end

__gfx__
00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
00700700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
00077000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
00077000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
00700700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
