; Example 102: Rounded Rectangle Demo - Arc Direction Examples
; 
; This file demonstrates the correct use of G2/G3 for rounded rectangle corners.
; 
; KEY INSIGHT: For a counter-clockwise toolpath around a rectangle,
; use G3 (CCW) at all corners to get OUTWARD-bulging fillets.
; Using G2 (CW) with positive R produces INWARD-curving arcs.
;
; The RS274/NGC standard defines:
; - Positive R = minor arc (shorter path, < 180°)
; - Negative R = major arc (longer path, > 180°)
;
; For outward-bulging corners on a CCW toolpath:
; - G3 (CCW) with positive R puts the center inside the corner
; - This causes the arc to curve away from the rectangle center

G21               ; Metric (mm)
G90               ; Absolute positioning
G17               ; XY plane

; Rectangle parameters
; Size: 100mm x 100mm (from -50 to +50 in both X and Y)
; Corner radius: 20mm

; ================================================================
; CORRECT: Counter-clockwise toolpath with G3 at corners
; Corners bulge OUTWARD (away from rectangle center)
; ================================================================

G0 Z5             ; Safe height
G0 X-50 Y-50      ; Start at bottom-left corner

; Comment: Starting CCW traversal
G1 X30 Y-50 F500  ; Bottom edge (moving right)

; Bottom-right corner - G3 produces outward bulge
G3 X50 Y-30 R20   ; Arc from (30,-50) to (50,-30)
                  ; Center at (~32,-32), arc curves toward exterior

G1 X50 Y30        ; Right edge (moving up)

; Top-right corner - G3 produces outward bulge  
G3 X30 Y50 R20    ; Arc from (50,30) to (30,50)

G1 X-30 Y50       ; Top edge (moving left)

; Top-left corner - G3 produces outward bulge
G3 X-50 Y30 R20   ; Arc from (-30,50) to (-50,30)

G1 X-50 Y-30      ; Left edge (moving down)

; Bottom-left corner - G3 produces outward bulge
G3 X-30 Y-50 R20  ; Arc from (-50,-30) to (-30,-50)

G1 X-50 Y-50      ; Return to start

G0 Z5             ; Retract

; ================================================================
; INCORRECT for outward corners: Same path with G2
; These corners would bulge INWARD (toward rectangle center)
; ================================================================

G0 X150 Y-50      ; Offset position for comparison

G1 X230 Y-50 F500 ; Bottom edge

; Bottom-right corner - G2 produces INWARD curve!
G2 X250 Y-30 R20  ; Arc curves toward interior
                  ; (Center at ~258,-58, outside the corner)

G1 X250 Y30       ; Right edge

G2 X230 Y50 R20   ; Top-right corner (inward)
G1 X170 Y50       ; Top edge
G2 X150 Y30 R20   ; Top-left corner (inward)
G1 X150 Y-30      ; Left edge
G2 X170 Y-50 R20  ; Bottom-left corner (inward)
G1 X150 Y-50      ; Return

G0 Z5
G0 X0 Y0          ; Return home
M30

; ================================================================
; Summary:
; - CCW path + G3 at corners = OUTWARD bulge (fillet effect)
; - CCW path + G2 at corners = INWARD bulge (chamfer effect)
; - CW path + G2 at corners = OUTWARD bulge
; - CW path + G3 at corners = INWARD bulge
;
; The arc direction (CW/CCW) relative to the toolpath direction
; determines whether the arc curves toward or away from the
; interior of the shape.
; ================================================================
