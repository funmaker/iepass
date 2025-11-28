pico-8 cartridge // http://www.pico-8.com
version 43
__lua__

printh("start===")
for fps=1,60 do
	printh("fps"..fps)
	_set_fps(fps)

	while not btn(0) do flip() end
	frame=0
	last = 0
	n = 0
	while btn(0) do
	 if btnp(0) and frame > 0 and n < 3 then
	 	printh(n.." "..(frame-last))
	 	last=frame
	 	n+=1
	 end
		
		flip()
		frame += 1
	end
end

