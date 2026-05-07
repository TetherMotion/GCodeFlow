; Example 010: Rapid Positioning
; Demonstrates G0 rapid moves for positioning

G90               ; Absolute mode
G21               ; Metric

; Rapid positioning pattern
G0 X0 Y0 Z50      ; Home position
G0 X50 Y0         ; Position 1
G0 Z5             ; Lower
G0 Z50            ; Retract
G0 X100 Y50       ; Position 2
G0 Z5
G0 Z50
G0 X50 Y100       ; Position 3
G0 Z5
G0 Z50
G0 X0 Y50         ; Position 4
G0 Z5
G0 Z50
G0 X0 Y0          ; Return home

M30
