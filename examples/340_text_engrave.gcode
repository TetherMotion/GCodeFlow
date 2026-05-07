; Example 340: Text Engraving
; Simple single-line text using basic stroke paths

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

; =============================================
; Text Engraving Settings
; =============================================
#<text_height> = 10     ; Character height in mm
#<text_x> = 10          ; Starting X position
#<text_y> = 80          ; Starting Y position
#<char_spacing> = 2     ; Space between characters
#<engrave_depth> = -1   ; Engraving depth

G0 Z10            ; Safe height

; =============================================
; Character Subroutines
; Each character drawn relative to bottom-left
; #1 = X start, #2 = Y start
; =============================================

; Letter G
o71 sub
  G0 X[#1 + 8] Y[#2 + 8]
  G1 Z#<engrave_depth> F100
  G3 X[#1 + 8] Y[#2 + 2] I[-8] J[-3] F300
  G3 X[#1] Y[#2 + 5] I0 J[3]
  G3 X[#1 + 4] Y[#2 + 10] I[4] J0
  G1 X[#1 + 8]
  G0 Z3
  G0 X[#1 + 4] Y[#2 + 5]
  G1 Z#<engrave_depth> F100
  G1 X[#1 + 8] F300
  G0 Z3
o71 endsub

; Letter C
o67 sub
  G0 X[#1 + 8] Y[#2 + 2]
  G1 Z#<engrave_depth> F100
  G3 X[#1] Y[#2 + 5] I0 J[3] F300
  G3 X[#1 + 8] Y[#2 + 8] I[8] J0
  G0 Z3
o67 endsub

; Letter O
o79 sub
  G0 X[#1 + 8] Y[#2 + 5]
  G1 Z#<engrave_depth> F100
  G3 I[-4] J0 F300   ; Full ellipse
  G0 Z3
o79 endsub

; Letter D
o68 sub
  G0 X[#1] Y[#2]
  G1 Z#<engrave_depth> F100
  G1 Y[#2 + 10] F300
  G1 X[#1 + 4]
  G3 X[#1 + 4] Y[#2] I0 J[-5]
  G1 X[#1]
  G0 Z3
o68 endsub

; Letter E
o69 sub
  G0 X[#1 + 8] Y[#2]
  G1 Z#<engrave_depth> F100
  G1 X[#1] F300
  G1 Y[#2 + 10]
  G1 X[#1 + 8]
  G0 Z3
  G0 X[#1] Y[#2 + 5]
  G1 Z#<engrave_depth> F100
  G1 X[#1 + 6] F300
  G0 Z3
o69 endsub

; Letter F
o70 sub
  G0 X[#1] Y[#2]
  G1 Z#<engrave_depth> F100
  G1 Y[#2 + 10] F300
  G1 X[#1 + 8]
  G0 Z3
  G0 X[#1] Y[#2 + 5]
  G1 Z#<engrave_depth> F100
  G1 X[#1 + 6] F300
  G0 Z3
o70 endsub

; Letter L
o76 sub
  G0 X[#1] Y[#2 + 10]
  G1 Z#<engrave_depth> F100
  G1 Y[#2] F300
  G1 X[#1 + 8]
  G0 Z3
o76 endsub

; Letter W
o87 sub
  G0 X[#1] Y[#2 + 10]
  G1 Z#<engrave_depth> F100
  G1 X[#1 + 2] Y[#2] F300
  G1 X[#1 + 4] Y[#2 + 6]
  G1 X[#1 + 6] Y[#2]
  G1 X[#1 + 8] Y[#2 + 10]
  G0 Z3
o87 endsub

; Number 0
o48 sub
  G0 X[#1 + 4] Y[#2]
  G1 Z#<engrave_depth> F100
  G3 X[#1] Y[#2 + 5] I0 J[5] F300
  G3 X[#1 + 4] Y[#2 + 10] I[4] J0
  G3 X[#1 + 8] Y[#2 + 5] I0 J[-5]
  G3 X[#1 + 4] Y[#2] I[-4] J0
  G0 Z3
o48 endsub

; Number 1
o49 sub
  G0 X[#1 + 2] Y[#2 + 8]
  G1 Z#<engrave_depth> F100
  G1 X[#1 + 4] Y[#2 + 10] F300
  G1 Y[#2]
  G0 Z3
  G0 X[#1 + 2] Y[#2]
  G1 Z#<engrave_depth> F100
  G1 X[#1 + 6] F300
  G0 Z3
o49 endsub

; =============================================
; Engrave "GCODE" 
; =============================================

#<x_pos> = #<text_x>

o71 call [#<x_pos>] [#<text_y>]              ; G
#<x_pos> = [#<x_pos> + 10 + #<char_spacing>]

o67 call [#<x_pos>] [#<text_y>]              ; C
#<x_pos> = [#<x_pos> + 10 + #<char_spacing>]

o79 call [#<x_pos>] [#<text_y>]              ; O
#<x_pos> = [#<x_pos> + 10 + #<char_spacing>]

o68 call [#<x_pos>] [#<text_y>]              ; D
#<x_pos> = [#<x_pos> + 10 + #<char_spacing>]

o69 call [#<x_pos>] [#<text_y>]              ; E
#<x_pos> = [#<x_pos> + 10 + #<char_spacing>]

; =============================================
; Engrave "FLOW" on second line
; =============================================

#<x_pos> = #<text_x>
#<text_y> = [#<text_y> - 15]

o70 call [#<x_pos>] [#<text_y>]              ; F
#<x_pos> = [#<x_pos> + 10 + #<char_spacing>]

o76 call [#<x_pos>] [#<text_y>]              ; L
#<x_pos> = [#<x_pos> + 10 + #<char_spacing>]

o79 call [#<x_pos>] [#<text_y>]              ; O
#<x_pos> = [#<x_pos> + 10 + #<char_spacing>]

o87 call [#<x_pos>] [#<text_y>]              ; W
#<x_pos> = [#<x_pos> + 10 + #<char_spacing>]

; =============================================
; Engrave "101" on third line
; =============================================

#<x_pos> = #<text_x>
#<text_y> = [#<text_y> - 15]

o49 call [#<x_pos>] [#<text_y>]              ; 1
#<x_pos> = [#<x_pos> + 10 + #<char_spacing>]

o48 call [#<x_pos>] [#<text_y>]              ; 0
#<x_pos> = [#<x_pos> + 10 + #<char_spacing>]

o49 call [#<x_pos>] [#<text_y>]              ; 1

G0 Z10            ; Retract
G0 X0 Y0          ; Return home
M30
