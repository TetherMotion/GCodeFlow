; Example 110: Helical Interpolation
; Demonstrates 3D helix for thread milling, ramping entry, etc.

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

; Helix parameters
; Center: (50, 50)
; Radius: 25mm
; Pitch: 5mm per revolution
; Total depth: 20mm

G0 Z10            ; Safe height
G0 X75 Y50        ; Start point (center_x + radius)

; Helical plunge - 4 revolutions descending
G1 Z0 F200        ; Touch surface
G2 X75 Y50 Z-5 I-25 J0 F300   ; Revolution 1
G2 X75 Y50 Z-10 I-25 J0       ; Revolution 2
G2 X75 Y50 Z-15 I-25 J0       ; Revolution 3
G2 X75 Y50 Z-20 I-25 J0       ; Revolution 4

; Final pass at full depth
G2 I-25 J0        ; Clean-up circle

; Exit helix - ascending
G2 X75 Y50 Z-15 I-25 J0 F400
G2 X75 Y50 Z-10 I-25 J0
G2 X75 Y50 Z-5 I-25 J0
G2 X75 Y50 Z0 I-25 J0

G0 Z10            ; Retract
G0 X0 Y0          ; Return home
M30
