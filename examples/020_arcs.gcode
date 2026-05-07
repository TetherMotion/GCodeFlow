; Example 020: Arc Interpolation
; Demonstrates G2 (CW) and G3 (CCW) arcs

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

G0 Z10            ; Safe height
G0 X0 Y50         ; Start position
G1 Z-5 F100       ; Plunge

; Clockwise semicircle (G2)
G2 X100 Y50 I50 J0 F300

; Counter-clockwise semicircle (G3)
G3 X0 Y50 I-50 J0 F300

; Full circle using G2
G0 X75 Y50
G2 I-25 J0        ; Full circle, 25mm radius

; Full circle using G3
G0 X125 Y50
G3 I-25 J0        ; Full circle, 25mm radius

; Quarter arcs
G0 X50 Y0
G3 X100 Y50 I0 J50 F300   ; 90 degree CCW arc
G3 X50 Y100 I-50 J0       ; 90 degree CCW arc
G3 X0 Y50 I0 J-50         ; 90 degree CCW arc
G3 X50 Y0 I50 J0          ; 90 degree CCW arc

G0 Z10            ; Retract
G0 X0 Y0          ; Return home
M30
