; Example 400: Complete Part Program
; Full machining program with multiple operations

%
O4000 (COMPLETE PART - BRACKET)
(Material: Aluminum 6061)
(Stock: 150x100x20mm)
(Author: GCodeFlow Example)
(Date: 2024)

; =============================================
; SETUP SECTION
; =============================================
G90 G54           ; Absolute, work offset 1
G21               ; Metric
G17               ; XY plane

; Machine parameters
#<safe_z> = 25
#<rapid_z> = 5
#<spindle_rpm> = 6000

; =============================================
; TOOL 1: 10mm END MILL - FACING & POCKETING
; =============================================
(MSG, Tool 1: 10mm End Mill)
T1 M6
G43 H1
S[#<spindle_rpm>] M3
G0 Z[#<safe_z>]

; ----- FACING PASS -----
; Face top surface
G0 X-5 Y-5
G0 Z1
G1 Z-0.5 F200     ; Light facing cut

; Zigzag facing pattern
#<y> = -5
o100 while [#<y> LT 105]
  G1 X155 F1500
  #<y> = [#<y> + 8]
  G1 Y[#<y>]
  G1 X-5
  #<y> = [#<y> + 8]
  G1 Y[#<y>]
o100 endwhile

G0 Z[#<rapid_z>]

; ----- RECTANGULAR POCKET -----
; Pocket: 80x50mm, 10mm deep, at (35, 25)
(MSG, Machining main pocket)

#<pocket_x> = 35
#<pocket_y> = 25
#<pocket_w> = 80
#<pocket_h> = 50
#<pocket_d> = -10
#<tool_r> = 5
#<stepover> = 6

; Depth stepping
#<z_step> = 3
#<current_z> = 0

o200 while [#<current_z> GT #<pocket_d>]
  #<current_z> = [#<current_z> - #<z_step>]
  o210 if [#<current_z> LT #<pocket_d>]
    #<current_z> = #<pocket_d>
  o210 endif
  
  ; Pocket clearing at this depth
  #<py> = [#<pocket_y> + #<tool_r>]
  G0 X[#<pocket_x> + #<tool_r>] Y[#<py>]
  G1 Z[#<current_z>] F200
  
  o220 while [#<py> LT [#<pocket_y> + #<pocket_h> - #<tool_r>]]
    G1 X[#<pocket_x> + #<pocket_w> - #<tool_r>] F1000
    #<py> = [#<py> + #<stepover>]
    G1 Y[#<py>]
    G1 X[#<pocket_x> + #<tool_r>]
    #<py> = [#<py> + #<stepover>]
    o225 if [#<py> LT [#<pocket_y> + #<pocket_h> - #<tool_r>]]
      G1 Y[#<py>]
    o225 endif
  o220 endwhile
  
  G0 Z[#<current_z> + 2]
o200 endwhile

; Finish pass around pocket perimeter
G0 X[#<pocket_x> + #<tool_r>] Y[#<pocket_y> + #<tool_r>]
G1 Z[#<pocket_d>] F200
G1 X[#<pocket_x> + #<pocket_w> - #<tool_r>] F800
G1 Y[#<pocket_y> + #<pocket_h> - #<tool_r>]
G1 X[#<pocket_x> + #<tool_r>]
G1 Y[#<pocket_y> + #<tool_r>]

G0 Z[#<safe_z>]
M5                ; Spindle stop

; =============================================
; TOOL 2: 6mm END MILL - CONTOUR & SLOTS
; =============================================
(MSG, Tool 2: 6mm End Mill)
T2 M6
G43 H2
S8000 M3
G0 Z[#<safe_z>]

; ----- OUTSIDE CONTOUR -----
; Rounded rectangle 140x90mm with R10 corners
(MSG, Cutting outside contour)

#<cont_x> = 5
#<cont_y> = 5
#<cont_w> = 140
#<cont_h> = 90
#<corner_r> = 10
#<cont_d> = -15

G0 X[#<cont_x> + #<corner_r>] Y[#<cont_y> - 5]  ; Approach from outside
G1 Z[#<cont_d>] F200

; Contour path with tabs
; Bottom edge
G1 X[#<cont_x> + #<cont_w> - #<corner_r>] Y[#<cont_y>] F600

; Bottom-right corner
G3 X[#<cont_x> + #<cont_w>] Y[#<cont_y> + #<corner_r>] I0 J[#<corner_r>]

; Right edge
G1 Y[#<cont_y> + #<cont_h> - #<corner_r>]

; Top-right corner
G3 X[#<cont_x> + #<cont_w> - #<corner_r>] Y[#<cont_y> + #<cont_h>] I[-#<corner_r>] J0

; Top edge
G1 X[#<cont_x> + #<corner_r>]

; Top-left corner
G3 X[#<cont_x>] Y[#<cont_y> + #<cont_h> - #<corner_r>] I0 J[-#<corner_r>]

; Left edge
G1 Y[#<cont_y> + #<corner_r>]

; Bottom-left corner
G3 X[#<cont_x> + #<corner_r>] Y[#<cont_y>] I[#<corner_r>] J0

G0 Z[#<safe_z>]

; ----- MOUNTING SLOTS -----
(MSG, Cutting mounting slots)

; Slot subroutine: #1=X, #2=Y, #3=length, #4=width
o300 sub
  #<slot_r> = [#4 / 2]
  G0 X[#1 + #<slot_r>] Y[#2]
  G1 Z-8 F200
  ; Slot profile
  G1 X[#1 + #3 - #<slot_r>] F600
  G3 X[#1 + #3 - #<slot_r>] Y[#2] I0 J[#<slot_r>]  ; Full semicircle
  G1 X[#1 + #<slot_r>]
  G3 X[#1 + #<slot_r>] Y[#2] I0 J[-#<slot_r>]
  G0 Z[#<rapid_z>]
o300 endsub

; Four mounting slots
o300 call [20] [15] [20] [8]    ; Bottom-left slot
o300 call [110] [15] [20] [8]   ; Bottom-right slot
o300 call [20] [80] [20] [8]    ; Top-left slot
o300 call [110] [80] [20] [8]   ; Top-right slot

G0 Z[#<safe_z>]
M5

; =============================================
; TOOL 3: 5mm DRILL - MOUNTING HOLES
; =============================================
(MSG, Tool 3: 5mm Drill)
T3 M6
G43 H3
S4000 M3
G0 Z[#<safe_z>]

; Drilling cycle
G0 X30 Y15
G81 R[#<rapid_z>] Z-20 F300    ; Drill cycle on
X120 Y15                        ; Hole 2
X120 Y80                        ; Hole 3
X30 Y80                         ; Hole 4
G80                             ; Cancel cycle

G0 Z[#<safe_z>]
M5

; =============================================
; TOOL 4: M6 TAP - THREAD HOLES
; =============================================
(MSG, Tool 4: M6 Tap)
T4 M6
G43 H4
S500 M3

; Rigid tapping cycle
G0 X75 Y50
G84 R[#<rapid_z>] Z-12 F500    ; Tap cycle (F = pitch * RPM)
G80

G0 Z[#<safe_z>]
M5

; =============================================
; TOOL 5: CHAMFER MILL - EDGE BREAK
; =============================================
(MSG, Tool 5: Chamfer Mill)
T5 M6
G43 H5
S5000 M3
G0 Z[#<safe_z>]

; Chamfer around pocket edge
G0 X[#<pocket_x>] Y[#<pocket_y>]
G1 Z-0.5 F200
G1 X[#<pocket_x> + #<pocket_w>] F800
G1 Y[#<pocket_y> + #<pocket_h>]
G1 X[#<pocket_x>]
G1 Y[#<pocket_y>]

G0 Z[#<safe_z>]
M5

; =============================================
; END PROGRAM
; =============================================
G0 Z[#<safe_z>]
G0 X0 Y0          ; Return to origin
M9                ; Coolant off
M30               ; Program end
%
