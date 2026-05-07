; Example 120: Simple Rectangular Pocket
; Zigzag pocket clearing pattern

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

; Pocket parameters
; Size: 60mm x 40mm
; Depth: 10mm
; Stepover: 5mm
; Start corner: (10, 10)

G0 Z10            ; Safe height
G0 X10 Y10        ; Start corner

; Depth pass 1: Z=-5
G1 Z-5 F100       ; Plunge

; Zigzag pattern
G1 X70 F500       ; Row 1
G1 Y15
G1 X10            ; Row 2
G1 Y20
G1 X70            ; Row 3
G1 Y25
G1 X10            ; Row 4
G1 Y30
G1 X70            ; Row 5
G1 Y35
G1 X10            ; Row 6
G1 Y40
G1 X70            ; Row 7
G1 Y45
G1 X10            ; Row 8
G1 Y50
G1 X70            ; Row 9

; Depth pass 2: Z=-10
G0 Z-4
G0 X10 Y10
G1 Z-10 F100

; Zigzag pattern again
G1 X70 F500
G1 Y15
G1 X10
G1 Y20
G1 X70
G1 Y25
G1 X10
G1 Y30
G1 X70
G1 Y35
G1 X10
G1 Y40
G1 X70
G1 Y45
G1 X10
G1 Y50
G1 X70

G0 Z10            ; Retract
G0 X0 Y0          ; Return home
M30
