pico-8 cartridge // http://www.pico-8.com
version 43
__lua__

function test_trace()
        p8rs.test("trace(nil)", trace(nil))
        p8rs.test("trace(\"Test Message\")", trace("Test Message"))
        p8rs.test("trace(nil, 0)", trace(nil, 0))
        p8rs.test("trace(nil, 3)", trace(nil, 3))
end

function top_level_fn()
        function nested_fn()
                test_trace()
        end
        nested_fn()
end

p8rs.test("trace(nil, 0)", trace(nil, 0))
p8rs.test("trace(nil, 1)", trace(nil, 1))
p8rs.test("trace(nil, 2)", trace(nil, 2))

test_trace()
top_level_fn()