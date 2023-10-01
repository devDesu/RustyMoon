function dumpupvals(f)
	for i=1,4 do -- VM design implies there can't be more 255 upvalues
       local value,name = debug.getupvalue(f,i)
       if value==nil then break end
       print(i, name, value)  --> 1  function: 0x419fd0  print
    end
end

local function x()
	local var1 = 100 --"test"
	
	local function test1()
		dumpupvals(test1)
		-- var1 = var1 .. "1"
		var1 = var1 + 1
		print(var1)
	end
	
	local function test2()
		-- var1 = var1 .. "2"
		var1 = var1 + 2
		print(var1)
	end
	
	print(var1)
	return test1, test2
end

local f1, f2 = x()

dumpupvals(f1)

f1()
f2()
f1()
f2()

dumpupvals(f2)