; Example 210: O-Code While Loops
; Demonstrates WHILE loop for repetitive operations

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

G0 Z10            ; Safe height

; =============================================
; Example 1: Simple counter loop
; Draw lines at Y=10, 20, 30, 40, 50
; =============================================

#1 = 10           ; Start Y position
o100 while [#1 LE 50]
  G0 X0 Y[#1]
  G1 Z-3 F100
  G1 X50 F500
  G0 Z5
  #1 = [#1 + 10]  ; Increment Y
o100 endwhile

; =============================================
; Example 2: Circular pattern using trig
; 8 holes in a circle
; =============================================

#<holes> = 8
#<radius> = 40
#<center_x> = 75
#<center_y> = 75
#<angle> = 0

o200 while [#<angle> LT 360]
  #<x> = [#<center_x> + #<radius> * COS[#<angle>]]
  #<y> = [#<center_y> + #<radius> * SIN[#<angle>]]
  
  G0 X[#<x>] Y[#<y>]
  G1 Z-5 F100     ; Drill
  G0 Z5           ; Retract
  
  #<angle> = [#<angle> + 360 / #<holes>]
o200 endwhile

; =============================================
; Example 3: Nested while loops - Grid pattern
; =============================================

#<start_x> = 10
#<start_y> = 100
#<spacing> = 10
#<cols> = 5
#<rows> = 3

#<row> = 0
o300 while [#<row> LT #<rows>]
  #<col> = 0
  o310 while [#<col> LT #<cols>]
    #<px> = [#<start_x> + #<col> * #<spacing>]
    #<py> = [#<start_y> + #<row> * #<spacing>]
    
    G0 X[#<px>] Y[#<py>]
    G1 Z-3 F100
    G0 Z3
    
    #<col> = [#<col> + 1]
  o310 endwhile
  #<row> = [#<row> + 1]
o300 endwhile

G0 Z10            ; Retract
G0 X0 Y0          ; Return home
M30
