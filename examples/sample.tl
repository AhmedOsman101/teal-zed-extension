#!/usr/bin/env tl
-- A sample covering the syntax the extension highlights and outlines.

local record Point
  x: number
  y: number
  metamethod __tostring: function(Point): string
end

local enum Color
  "red"
  "green"
  "blue"
end

local type Pair<T> = { T, T }

local function distance(a: Point, b: Point): number
  local dx = a.x - b.x
  local dy = a.y - b.y
  return math.sqrt(dx * dx + dy * dy)
end

local p: Point = { x = 0, y = 0 }
local q: Point = { x = 3, y = 4 }

for i = 1, 3 do
  print(i, distance(p, q))
end

while distance(p, q) > 0 do
  break
end

repeat
  local unused = 1
until true

if distance(p, q) > 3 then
  print("far")
elseif distance(p, q) == 5 then
  print("five")
else
  print("near")
end

-- Deliberate type error for diagnostics.
local label: string = distance(p, q)
local pair: Pair<number> = { 1, 2 }
print(label, "red" as Color, pair[1])
