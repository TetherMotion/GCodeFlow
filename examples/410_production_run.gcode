; Example 410: Multi-Part Production Run
; Production run with part counter and statistics

%
O4100 (PRODUCTION RUN - 25 PARTS)

; =============================================
; PRODUCTION CONFIGURATION
; =============================================
#<total_parts> = 25
#<current_part> = 0
#<good_parts> = 0

; Part spacing on fixture
#<part_spacing_x> = 60
#<part_spacing_y> = 60
#<parts_per_row> = 5

; Part dimensions
#<part_w> = 50
#<part_h> = 50
#<part_d> = 5

; =============================================
; SETUP
; =============================================
G90 G54
G21
G17

T1 M6             ; 8mm end mill
G43 H1
S5000 M3

; =============================================
; PRODUCTION LOOP
; =============================================
o100 while [#<current_part> LT #<total_parts>]
  
  ; Calculate part position
  #<col> = [#<current_part> MOD #<parts_per_row>]
  #<row> = [FIX[#<current_part> / #<parts_per_row>]]
  
  #<base_x> = [#<col> * #<part_spacing_x>]
  #<base_y> = [#<row> * #<part_spacing_y>]
  
  (MSG, Part #<current_part> at X#<base_x> Y#<base_y>)
  
  ; ----- MACHINE ONE PART -----
  ; Rapid to part start
  G0 Z25
  G0 X[#<base_x>] Y[#<base_y>]
  
  ; Cut outer contour
  G0 Z2
  G1 Z[-#<part_d>] F200
  
  ; Rectangle contour
  G1 X[#<base_x> + #<part_w>] F1000
  G1 Y[#<base_y> + #<part_h>]
  G1 X[#<base_x>]
  G1 Y[#<base_y>]
  
  ; Center pocket
  #<pocket_margin> = 10
  G0 Z2
  G0 X[#<base_x> + #<pocket_margin>] Y[#<base_y> + #<pocket_margin>]
  G1 Z[-#<part_d> + 2] F200
  
  ; Pocket pattern
  #<py> = [#<base_y> + #<pocket_margin>]
  o110 while [#<py> LT [#<base_y> + #<part_h> - #<pocket_margin>]]
    G1 X[#<base_x> + #<part_w> - #<pocket_margin>] F800
    G1 Y[#<py> + 5]
    G1 X[#<base_x> + #<pocket_margin>]
    #<py> = [#<py> + 10]
    G1 Y[#<py>]
  o110 endwhile
  
  ; Center mark (identification)
  G0 Z2
  G0 X[#<base_x> + #<part_w>/2] Y[#<base_y> + #<part_h>/2]
  G1 Z[-#<part_d> + 3] F100
  ; Engrave part number
  #<digit> = #<current_part>
  ; Simple cross mark for now
  G1 X[#<base_x> + #<part_w>/2 + 3] Y[#<base_y> + #<part_h>/2]
  G1 X[#<base_x> + #<part_w>/2 - 3]
  G0 X[#<base_x> + #<part_w>/2] Y[#<base_y> + #<part_h>/2 + 3]
  G1 Y[#<base_y> + #<part_h>/2 - 3]
  
  ; Update counters
  #<current_part> = [#<current_part> + 1]
  #<good_parts> = [#<good_parts> + 1]
  
  ; Retract
  G0 Z25
  
o100 endwhile

; =============================================
; PRODUCTION SUMMARY
; =============================================
G0 Z50
G0 X0 Y0
(MSG, Production complete)
(MSG, Total parts: #<total_parts>)
(MSG, Good parts: #<good_parts>)

M5
M30
%
