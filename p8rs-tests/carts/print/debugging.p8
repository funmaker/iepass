pico-8 cartridge // http://www.pico-8.com
version 43
__lua__
-------------------------
-- helpers / utilities --
-------------------------

-- reset video + text state to a known baseline
function reset_env()
  cls()
  camera(0,0)
  clip()

  -- text flags / metrics defaults:
  -- 0x5f36: we force wrap off (bit7=0), scroll on (bit6=0)
  poke(0x5f36, 0x00)

  -- 0x5f58: print attribute flags (wide/tall etc) – clear
  poke(0x5f58, 0x00)

  -- 0x5f59: high nibble = char_h, low nibble = char_w
  -- default font: 4×6
  --   high = 6 (0x6), low = 4 (0x4) → 0x64
  poke(0x5f59, 0x64)

  -- cursor + margin
  cursor(0,0)
end

-- simple background encoding y into color on x=0
-- so scrolling can be detected by reading that column
function init_background()
  cls()
  for y=0,127 do
    local c = y \ 8  -- 0..15, each color = 8px band
    pset(0,y,c)
  end
end

-- sample the colors of a vertical stripe at x
-- result is [128] array of 0..15
function sample_vstripe(x)
  local t = {}
  for y=0,127 do
    add(t, pget(x,y))
  end
  return t
end

-- capture core text / cursor state (no pixels)
function snapshot_state(label)
  local attr_wh = peek(0x5f59)
  local attr_flags = peek(0x5f58)
  return {
    label       = label,
    cursor_x    = peek(0x5f26),
    cursor_y    = peek(0x5f27),
    margin_left = peek(0x5f24),
    flags_5f36  = peek(0x5f36),

    attr_flags  = attr_flags,
    char_w      = band(attr_wh,0x0f),
    char_h      = flr(attr_wh/16),

    wide_attr   = band(attr_flags,0x01) ~= 0,
    tall_attr   = band(attr_flags,0x02) ~= 0
  }
end

-- helper: one step’s before/after snapshots + stripes
function capture_step(label_before, label_after)
  local before_state  = snapshot_state(label_before)
  local stripe_before = sample_vstripe(0)

  local after_state   = snapshot_state(label_after)
  local stripe_after  = sample_vstripe(0)

  return {
    before_state  = before_state,
    after_state   = after_state,
    stripe_before = stripe_before,
    stripe_after  = stripe_after
  }
end

-----------------
-- test suite  --
-----------------

test_data = {}

-- 1) coordinate mode followed by console mode
--    reproduces your example and some variations
function test_console_vs_xy()
  local t = { name="console_vs_xy", cases={} }

  -- base case: your example
  reset_env()
  init_background()

  print("xx", 0, 115)
  local cursor1 = { x=peek(0x5f26), y=peek(0x5f27) }

  print("yy") -- console mode, potential scroll
  local cursor2 = { x=peek(0x5f26), y=peek(0x5f27) }

  local s = capture_step("after_xy", "after_console")

  add(t.cases, {
    desc     = "example_xy_115",
    cursor1  = cursor1,
    cursor2  = cursor2,
    step     = s
  })

  -- tweak y to probe threshold more finely
  for y=108,126,2 do
    reset_env()
    init_background()

    print("xx", 0, y)
    local c1 = { x=peek(0x5f26), y=peek(0x5f27) }

    print("yy")
    local c2 = { x=peek(0x5f26), y=peek(0x5f27) }

    local s2 = capture_step("after_xy_y"..y, "after_console_y"..y)

    add(t.cases, {
      desc    = "xy_then_console_y_"..y,
      y       = y,
      cursor1 = c1,
      cursor2 = c2,
      step    = s2
    })
  end

  add(test_data, t)
end

-- 2) console-mode threshold sweep
--    cursor set via cursor(), print() without x,y
--    we sweep y near the bottom to see when scroll kicks in
function test_threshold_sweep()
  local t = { name="threshold_sweep", cases={} }

  for y=110,127 do
    reset_env()
    init_background()

    cursor(0,y)
    local before_state = snapshot_state("before_print_y_"..y)
    local stripe_before = sample_vstripe(0)

    print("xx") -- console mode, implicit newline

    local after_state = snapshot_state("after_print_y_"..y)
    local stripe_after = sample_vstripe(0)

    add(t.cases, {
      y = y,
      before_state  = before_state,
      after_state   = after_state,
      stripe_before = stripe_before,
      stripe_after  = stripe_after
    })
  end

  add(test_data, t)
end

-- 3) explicit vs implicit newline, including \0 terminator
--    goal: see whether scroll is tied to newline or to end-of-call
function test_explicit_vs_implicit()
  local t = { name="explicit_vs_implicit", cases={} }

  local ys = {110, 118, 122}

  for i=1,#ys do
    local y = ys[i]

    -- a) explicit newline inside string
    reset_env()
    init_background()
    cursor(0,y)

    print("a\nb")
    local s1 = capture_step("exp_before_y"..y, "exp_after_y"..y)

    -- b) two separate print() calls, each console-mode
    reset_env()
    init_background()
    cursor(0,y)

    print("a")
    local mid_state  = snapshot_state("after_first_print_y"..y)
    local mid_stripe = sample_vstripe(0)
    print("b")
    local s2 = capture_step("mid_y"..y, "after_two_prints_y"..y)
    s2.mid_state  = mid_state
    s2.mid_stripe = mid_stripe

    -- c) explicit \n but with \0 to suppress implicit newline
    reset_env()
    init_background()
    cursor(0,y)

    print("a\nb\0")
    local s3 = capture_step("exp0_before_y"..y, "exp0_after_y"..y)

    add(t.cases, {
      y      = y,
      a_explicit      = s1,
      b_two_calls     = s2,
      c_explicit_zero = s3
    })
  end

  add(test_data, t)
end

-- 4) tall-mode behaviour:
--    - tall for entire line
--    - tall then cleared before end of line
--    - tall on first line only vs second line
function test_tall_mode()
  local t = { name="tall_mode", cases={} }

  local ys = {108, 116, 120}

  for i=1,#ys do
    local y = ys[i]

    -- case 1: tall whole line, implicit newline
    reset_env()
    init_background()
    cursor(0,y)
    print("\^tAA")  -- tall for both glyphs
    local case1 = capture_step("tall_full_before_y"..y, "tall_full_after_y"..y)

    -- case 2: tall then normal on same line
    reset_env()
    init_background()
    cursor(0,y)
    print("\^tA\^-tA")
    local case2 = capture_step("tall_partial_before_y"..y, "tall_partial_after_y"..y)

    -- case 3: tall, explicit newline, then normal second line
    reset_env()
    init_background()
    cursor(0,y)
    print("\^tA\nB")
    local case3 = capture_step("tall_exp_before_y"..y, "tall_exp_after_y"..y)

    -- case 4: tall on second line only
    reset_env()
    init_background()
    cursor(0,y)
    print("A\n\^tB")
    local case4 = capture_step("tall_second_before_y"..y, "tall_second_after_y"..y)

    add(t.cases, {
      y       = y,
      tall_full      = case1,
      tall_partial   = case2,
      tall_firstline = case3,
      tall_secondline= case4
    })
  end

  add(test_data, t)
end

-- 5) custom char_h via ^y – tests interaction of advance vs scroll
function test_custom_char_h()
  local t = { name="custom_char_h", cases={} }

  local ys = {110, 118, 122}
  local heights = {4, 6, 8, 10, 12}

  for i=1,#ys do
    local y = ys[i]
    for j=1,#heights do
      local h = heights[j]

      reset_env()
      init_background()
      cursor(0,y)

      -- set advance height with ^y, then print implicit newline
      -- form: "^y" <number> "A"
      local s = "\^y"..h.."A"
      print(s)

      local case = capture_step("cust_h_before_y"..y.."_h"..h,
                                "cust_h_after_y"..y.."_h"..h)

      case.y = y
      case.h = h

      add(t.cases, case)
    end
  end

  add(test_data, t)
end

-- 6) wrapping behaviour:
--    - wrapping off vs on
--    - custom wrap boundary via ^r
--    we look at how many lines we get and whether a scroll occurs
function test_wrap()
  local t = { name="wrap_mode", cases={} }

  -- helper string long enough to potentially wrap
  local long = "abcdefghijklmno"

  local ys = {110, 118, 122}

  for i=1,#ys do
    local y = ys[i]

    -- (a) wrapping off (default)
    reset_env()
    init_background()
    cursor(0,y)
    print(long)
    local a = capture_step("wrapoff_before_y"..y, "wrapoff_after_y"..y)

    -- (b) wrapping on (bit7=1), default border
    reset_env()
    init_background()
    cursor(0,y)
    poke(0x5f36, peek(0x5f36) | 0x80)
    print(long)
    local b = capture_step("wrapon_before_y"..y, "wrapon_after_y"..y)

    -- (c) wrapping on, narrow ^r border
    reset_env()
    init_background()
    cursor(0,y)
    poke(0x5f36, peek(0x5f36) | 0x80)
    print("\^r4"..long)  -- wrap column at x = 4*4 = 16px
    local c = capture_step("wrapon_r4_before_y"..y, "wrapon_r4_after_y"..y)

    add(t.cases,{
      y      = y,
      wrapoff = a,
      wrapon  = b,
      wrapon_r4 = c
    })
  end

  add(test_data, t)
end

-- 7) probing "pre-print scroll" – starting with cursor very low
--    goal: see if a scroll happens *before* any glyph is drawn on the new line
--    we can't see mid-call, but we can bias the background to show
--    exactly how many pixels moved when a single short print happens
function test_preline_scroll_feel()
  local t = { name="preline_scroll_feel", cases={} }

  -- use tiny strings so single-line behaviour is isolated
  local ys = {120, 122, 124, 126}

  for i=1,#ys do
    local y = ys[i]

    reset_env()
    init_background()
    cursor(0,y)

    -- single console-mode print of one char; if pico-8 is doing a
    -- "pre-line scroll" because this line wouldn't fit, we’ll see
    -- full-frame scroll in stripe diff even though only one glyph is drawn
    local before_state = snapshot_state("preline_before_y"..y)
    local stripe_before = sample_vstripe(0)

    print("x")

    local after_state = snapshot_state("preline_after_y"..y)
    local stripe_after = sample_vstripe(0)

    add(t.cases, {
      y             = y,
      before_state  = before_state,
      after_state   = after_state,
      stripe_before = stripe_before,
      stripe_after  = stripe_after
    })
  end

  add(test_data, t)
end

-----------------
-- entry point --
-----------------

function run_all_tests()
  test_console_vs_xy()
  test_threshold_sweep()
  test_explicit_vs_implicit()
  test_tall_mode()
  test_custom_char_h()
  test_wrap()
  test_preline_scroll_feel()
end
run_all_tests()
p8rs.test("Data", test_data)
