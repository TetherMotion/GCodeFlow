; Example 101: Circle Patterns
; Various circle sizes and patterns

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

G0 Z10            ; Safe height

; Circle 1: R=10 at (25, 25)
G0 X35 Y25        ; Start point (center_x + radius)
G1 Z-3 F100       ; Plunge
G2 I-10 J0 F400   ; Full CW circle

; Circle 2: R=15 at (75, 25)
G0 Z5
G0 X90 Y25
G1 Z-3 F100
G3 I-15 J0 F400   ; Full CCW circle

; Circle 3: R=20 at (25, 75)
G0 Z5
G0 X45 Y75
G1 Z-3 F100
G2 I-20 J0 F400

; Circle 4: R=25 at (75, 75)
G0 Z5
G0 X100 Y75
G1 Z-3 F100
G3 I-25 J0 F400

; Concentric circles at center (50, 125)
G0 Z5
G0 X55 Y125
G1 Z-3 F100
G2 I-5 J0 F400    ; R=5
G0 X60 Y125
G2 I-10 J0        ; R=10
G0 X65 Y125
G2 I-15 J0        ; R=15
G0 X70 Y125
G2 I-20 J0        ; R=20
G0 X75 Y125
G2 I-25 J0        ; R=25

G0 Z10            ; Retract
G0 X0 Y0          ; Return home
M30
