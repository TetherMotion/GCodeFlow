; Example 230: O-Code Conditionals
; Demonstrates IF/ELSEIF/ELSE/ENDIF control flow

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

G0 Z10            ; Safe height

; =============================================
; Example 1: Simple IF/ELSE
; =============================================

#<size> = 30

o100 if [#<size> GT 25]
  ; Large size - cut outer rectangle
  G0 X10 Y10
  G1 Z-3 F100
  G1 X[10 + #<size>] F500
  G1 Y[10 + #<size>]
  G1 X10
  G1 Y10
  G0 Z5
o100 else
  ; Small size - just mark center
  G0 X[10 + #<size>/2] Y[10 + #<size>/2]
  G1 Z-3 F100
  G0 Z5
o100 endif

; =============================================
; Example 2: IF/ELSEIF chain - shape selection
; =============================================

#<shape> = 2      ; 1=square, 2=circle, 3=triangle

o200 if [#<shape> EQ 1]
  ; Draw square
  G0 X60 Y10
  G1 Z-3 F100
  G1 X80 F500
  G1 Y30
  G1 X60
  G1 Y10
  G0 Z5
o200 elseif [#<shape> EQ 2]
  ; Draw circle
  G0 X80 Y20      ; Start at right edge
  G1 Z-3 F100
  G2 I-10 J0 F400 ; Full circle R=10
  G0 Z5
o200 elseif [#<shape> EQ 3]
  ; Draw triangle
  G0 X60 Y10
  G1 Z-3 F100
  G1 X80 Y10 F500
  G1 X70 Y27
  G1 X60 Y10
  G0 Z5
o200 else
  ; Unknown shape - do nothing
  (MSG, Unknown shape type)
o200 endif

; =============================================
; Example 3: Comparison operators
; =============================================

#<value> = 50

; LT (less than)
o300 if [#<value> LT 100]
  G0 X10 Y50
  G1 Z-2 F100
  G0 Z5
o300 endif

; LE (less than or equal)
o310 if [#<value> LE 50]
  G0 X30 Y50
  G1 Z-2 F100
  G0 Z5
o310 endif

; EQ (equal)
o320 if [#<value> EQ 50]
  G0 X50 Y50
  G1 Z-2 F100
  G0 Z5
o320 endif

; NE (not equal)
o330 if [#<value> NE 100]
  G0 X70 Y50
  G1 Z-2 F100
  G0 Z5
o330 endif

; GE (greater than or equal)
o340 if [#<value> GE 50]
  G0 X90 Y50
  G1 Z-2 F100
  G0 Z5
o340 endif

; GT (greater than)
o350 if [#<value> GT 0]
  G0 X110 Y50
  G1 Z-2 F100
  G0 Z5
o350 endif

; =============================================
; Example 4: Logical operators
; =============================================

#<a> = 1
#<b> = 0

; AND
o400 if [[#<a> EQ 1] AND [#<b> EQ 0]]
  G0 X10 Y70
  G1 Z-2 F100
  G0 Z5
o400 endif

; OR
o410 if [[#<a> EQ 1] OR [#<b> EQ 1]]
  G0 X30 Y70
  G1 Z-2 F100
  G0 Z5
o410 endif

G0 Z10            ; Retract
G0 X0 Y0          ; Return home
M30
