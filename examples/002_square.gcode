; Example 002: Simple Square
; 100mm x 100mm square path

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

G0 Z10            ; Safe height
G0 X0 Y0          ; Start corner
G1 Z-5 F100       ; Plunge

; Cut square clockwise
G1 X100 F500      ; Right
G1 Y100           ; Up
G1 X0             ; Left
G1 Y0             ; Down (back to start)

G0 Z10            ; Retract
G0 X0 Y0          ; Return to origin
M30
