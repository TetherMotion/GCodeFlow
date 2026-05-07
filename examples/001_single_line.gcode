; Example 001: Single Linear Move
; Demonstrates a basic G1 feed move

G90         ; Absolute positioning
G21         ; Metric units (mm)
G0 X0 Y0 Z5 ; Start position
G1 X100 Y100 Z-5 F500 ; Linear move with feed rate
M30
