; Example 220: O-Code Repeat Loops
; Demonstrates REPEAT for fixed iteration counts

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

G0 Z10            ; Safe height

; =============================================
; Example 1: Simple repeat - 5 drilling operations
; =============================================

#<x> = 10
o100 repeat [5]
  G0 X[#<x>] Y10
  G1 Z-10 F100    ; Drill
  G0 Z5           ; Retract
  #<x> = [#<x> + 15]
o100 endrepeat

; =============================================
; Example 2: Nested repeats - Create a 3x4 grid
; =============================================

#<base_y> = 40
o200 repeat [4]   ; 4 rows
  #<base_x> = 10
  o210 repeat [3] ; 3 columns
    G0 X[#<base_x>] Y[#<base_y>]
    G1 Z-5 F100
    ; Small square at each position
    G1 X[#<base_x> + 5] F500
    G1 Y[#<base_y> + 5]
    G1 X[#<base_x>]
    G1 Y[#<base_y>]
    G0 Z5
    #<base_x> = [#<base_x> + 20]
  o210 endrepeat
  #<base_y> = [#<base_y> + 15]
o200 endrepeat

; =============================================
; Example 3: Repeat with depth stepping
; Progressive depth cuts
; =============================================

G0 X80 Y30
#<depth> = 0
#<step> = 2

o300 repeat [5]   ; 5 passes
  #<depth> = [#<depth> - #<step>]
  G1 Z[#<depth>] F100
  ; Cut a circle at this depth
  G2 I-15 J0 F400
o300 endrepeat

G0 Z10            ; Retract
G0 X0 Y0          ; Return home
M30
