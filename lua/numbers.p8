pico-8 cartridge // http://www.pico-8.com
version 8

__lua__
printh("Hex,Sin,Cos,atan2(x, 0.5),Sqrt,x^2,2^x,Dec")
for val = 0,1,0x0.0001 do
	printh(
		tostr(val, true) .. "," ..
		tostr(sin(val), true) .. "," ..
		tostr(cos(val), true) .. "," ..
		tostr(atan2(val, 0.5), true) .. "," ..
		tostr(sqrt(val), true) .. "," ..
		tostr(val^2, true) .. "," ..
		tostr(2^val, true) .. "," ..
		tostr(val)
	)
end
