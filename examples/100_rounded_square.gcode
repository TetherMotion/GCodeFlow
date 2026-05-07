; Example 100: Rounded Square
; Square with rounded corners using arcs

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

; Parameters
; Square: 80mm x 80mm
; Corner radius: 10mm
; Center at: 50, 50

G0 Z10            ; Safe height
G0 X10 Y0         ; Start position (bottom left + radius)
G1 Z-5 F100       ; Plunge

; Bottom edge to bottom-right corner
G1 X70 F500       ; Straight to corner start

; Bottom-right corner (90 deg CCW)
G3 X80 Y10 I0 J10

; Right edge to top-right corner
G1 Y70            ; Straight up

; Top-right corner
G3 X70 Y80 I-10 J0

; Top edge to top-left corner
G1 X10            ; Straight left

; Top-left corner
G3 X0 Y70 I0 J-10

; Left edge to bottom-left corner
G1 Y10            ; Straight down

; Bottom-left corner
G3 X10 Y0 I10 J0  ; Back to start

G0 Z10            ; Retract
G0 X0 Y0          ; Return home
M30
