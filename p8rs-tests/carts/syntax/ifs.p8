
if true then
  p8rs.test("standard then 1")
else
  p8rs.test("standard else 1")
end

if false then
  p8rs.test("standard then 2")
else
  p8rs.test("standard else 2")
end

if(true) p8rs.test("short then 1") else p8rs.test("short else 1")
if(false) p8rs.test("short then 2") else p8rs.test("short else 2")
if(false) p8rs.test("short then 3") elseif true then p8rs.test("short elseif 3") else p8rs.test("short else 3")

if(false) if(false) p8rs.test("nested then 1") else p8rs.test("nested inner 1") else p8rs.test("nested outer 1")
if(false) if(true) p8rs.test("nested then 2") else p8rs.test("nested inner 2") else p8rs.test("nested outer 2")
if(true) if(false) p8rs.test("nested then 3") else p8rs.test("nested inner 3") else p8rs.test("nested outer 3")
if(true) if(true) p8rs.test("nested then 4") else p8rs.test("nested inner 4") else p8rs.test("nested outer 4")
