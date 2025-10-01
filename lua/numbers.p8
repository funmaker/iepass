pico-8 cartridge // http://www.pico-8.com
version 8

__lua__
printh("Hex,Sin,Cos,atan2-ne,atan2-nw,atan2-sw,atan2-se,Sqrt,x^2,2^x,ToDecimal")
for val = 0,1,0x0.0001 do
	printh(
		tostr(val, true) .. "," ..
		tostr(sin(val), true) .. "," ..
		tostr(cos(val), true) .. "," ..
		tostr(atan2( 1.0 - val, -val), true) .. "," ..
		tostr(atan2(-val,  val - 1.0), true) .. "," ..
		tostr(atan2( val - 1.0,  val), true) .. "," ..
		tostr(atan2( val,  1.0 - val), true) .. "," ..
		tostr(sqrt(val), true) .. "," ..
		tostr(val^2, true) .. "," ..
		tostr(2^val, true) .. "," ..
		tostr(val)
	)
end
