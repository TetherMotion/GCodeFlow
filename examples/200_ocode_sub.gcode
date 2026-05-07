; Example 200: O-Code Subroutines
; Demonstrates defining and calling subroutines

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

; Define a subroutine to cut a small square
; Parameters: #1=X start, #2=Y start, #3=size
o100 sub
  G0 X[#1] Y[#2] Z5
  G1 Z-3 F100
  G1 X[#1 + #3] F500
  G1 Y[#2 + #3]
  G1 X[#1]
  G1 Y[#2]
  G0 Z5
o100 endsub

; Define a subroutine to cut a triangle
; Parameters: #1=X center, #2=Y center, #3=size
o200 sub
  #10 = [#3 / 2]           ; Half size
  G0 X[#1 - #10] Y[#2 - #10 * 0.577] Z5
  G1 Z-3 F100
  G1 X[#1 + #10] Y[#2 - #10 * 0.577] F500
  G1 X[#1] Y[#2 + #10 * 1.155]
  G1 X[#1 - #10] Y[#2 - #10 * 0.577]
  G0 Z5
o200 endsub

G0 Z10            ; Safe height

; Cut squares at various positions
o100 call [10] [10] [20]   ; Square at (10,10), size 20
o100 call [50] [10] [15]   ; Square at (50,10), size 15
o100 call [80] [10] [25]   ; Square at (80,10), size 25

; Cut triangles
o200 call [25] [60] [30]   ; Triangle at (25,60), size 30
o200 call [75] [60] [20]   ; Triangle at (75,60), size 20

G0 Z10            ; Retract
G0 X0 Y0          ; Return home
M30
