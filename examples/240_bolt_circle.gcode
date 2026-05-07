; Example 240: Parametric Bolt Circle
; Uses O-code to create bolt circle patterns

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

; =============================================
; Bolt Circle Subroutine
; Parameters:
;   #1 = Center X
;   #2 = Center Y
;   #3 = Bolt circle radius
;   #4 = Number of holes
;   #5 = Drill depth
;   #6 = Starting angle (degrees)
; =============================================
o100 sub
  #<idx> = 0
  
  o110 while [#<idx> LT #4]
    ; Calculate angle for this hole
    #<angle> = [#6 + #<idx> * 360 / #4]
    
    ; Calculate X, Y position using trig
    #<hx> = [#1 + #3 * COS[#<angle>]]
    #<hy> = [#2 + #3 * SIN[#<angle>]]
    
    ; Rapid to position
    G0 X[#<hx>] Y[#<hy>]
    
    ; Drill cycle
    G1 Z[#5] F100
    G0 Z5
    
    #<idx> = [#<idx> + 1]
  o110 endwhile
o100 endsub

; =============================================
; Main Program
; =============================================

G0 Z10            ; Safe height

; First bolt circle: 6 holes, R=25, centered at (40, 40)
(MSG, Drilling 6-hole bolt circle)
o100 call [40] [40] [25] [6] [-10] [0]

; Second bolt circle: 8 holes, R=30, centered at (110, 40), start at 22.5 deg
(MSG, Drilling 8-hole bolt circle)
o100 call [110] [40] [30] [8] [-10] [22.5]

; Third bolt circle: 4 holes, R=20, centered at (40, 110)
(MSG, Drilling 4-hole bolt circle)
o100 call [40] [110] [20] [4] [-10] [45]

; Fourth bolt circle: 12 holes, R=35, centered at (110, 110)
(MSG, Drilling 12-hole bolt circle)
o100 call [110] [110] [35] [12] [-8] [0]

; Add center holes
G0 X40 Y40
G1 Z-15 F100
G0 Z5
G0 X110 Y40
G1 Z-15 F100
G0 Z5
G0 X40 Y110
G1 Z-15 F100
G0 Z5
G0 X110 Y110
G1 Z-15 F100
G0 Z5

G0 Z10            ; Retract
G0 X0 Y0          ; Return home
M30
