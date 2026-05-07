; Example 300: Nested Control Flow
; Complex nested loops and conditionals

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

G0 Z10            ; Safe height

; =============================================
; Parametric Grid with Conditional Patterns
; Creates different shapes based on position
; =============================================

; Shape subroutines
o10 sub           ; Square shape
  G1 Z-3 F100
  G1 X[#<px> + 8] F500
  G1 Y[#<py> + 8]
  G1 X[#<px>]
  G1 Y[#<py>]
  G0 Z3
o10 endsub

o20 sub           ; Circle shape
  G0 X[#<px> + 4] Y[#<py> + 4]
  G1 Z-3 F100
  G2 I-4 J0 F400
  G0 Z3
o20 endsub

o30 sub           ; Triangle shape
  G1 Z-3 F100
  G1 X[#<px> + 8] Y[#<py>] F500
  G1 X[#<px> + 4] Y[#<py> + 7]
  G1 X[#<px>] Y[#<py>]
  G0 Z3
o30 endsub

o40 sub           ; Diamond shape
  G0 X[#<px> + 4] Y[#<py>]
  G1 Z-3 F100
  G1 X[#<px> + 8] Y[#<py> + 4] F500
  G1 X[#<px> + 4] Y[#<py> + 8]
  G1 X[#<px>] Y[#<py> + 4]
  G1 X[#<px> + 4] Y[#<py>]
  G0 Z3
o40 endsub

; Main nested loop
#<rows> = 5
#<cols> = 6
#<spacing> = 15

#<row> = 0
o100 while [#<row> LT #<rows>]
  
  #<col> = 0
  o110 while [#<col> LT #<cols>]
    
    ; Calculate position
    #<px> = [10 + #<col> * #<spacing>]
    #<py> = [10 + #<row> * #<spacing>]
    
    ; Calculate pattern based on position
    #<pattern> = [[#<row> + #<col>] MOD 4]
    
    G0 X[#<px>] Y[#<py>]
    
    ; Choose shape based on pattern
    o200 if [#<pattern> EQ 0]
      o10 call    ; Square
    o200 elseif [#<pattern> EQ 1]
      o20 call    ; Circle
    o200 elseif [#<pattern> EQ 2]
      o30 call    ; Triangle
    o200 else
      o40 call    ; Diamond
    o200 endif
    
    #<col> = [#<col> + 1]
  o110 endwhile
  
  #<row> = [#<row> + 1]
o100 endwhile

; =============================================
; Recursive-style depth pattern
; Each level smaller than previous
; =============================================

o500 sub          ; Recursive square pattern
  ; #1 = X, #2 = Y, #3 = size, #4 = depth level
  
  o510 if [#4 GT 0]
    ; Draw square at current position/size
    G0 X[#1] Y[#2]
    G1 Z[-3 - #4] F100
    G1 X[#1 + #3] F500
    G1 Y[#2 + #3]
    G1 X[#1]
    G1 Y[#2]
    G0 Z5
    
    ; Recursive call with smaller size (simulated with while)
    o520 if [#3 GT 10]
      #<newsize> = [#3 * 0.6]
      #<offset> = [[#3 - #<newsize>] / 2]
      o500 call [#1 + #<offset>] [#2 + #<offset>] [#<newsize>] [#4 - 1]
    o520 endif
  o510 endif
o500 endsub

; Call recursive pattern
G0 Z10
o500 call [110] [40] [50] [4]

G0 Z10            ; Retract
G0 X0 Y0          ; Return home
M30
