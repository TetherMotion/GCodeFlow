; Example 500: 5-Axis Impeller Blade
; Simulated 5-axis machining (A and B rotary)

%
O5000 (5-AXIS IMPELLER BLADE)

; =============================================
; IMPELLER PARAMETERS
; =============================================
#<num_blades> = 6
#<hub_dia> = 30
#<outer_dia> = 80
#<blade_height> = 25
#<blade_twist> = 45       ; degrees of twist root to tip

; Blade profile
#<inlet_angle> = 30       ; Leading edge angle
#<outlet_angle> = 60      ; Trailing edge angle
#<thickness> = 3          ; Blade thickness

; =============================================
; SETUP - 5-AXIS MACHINE
; =============================================
G90 G54
G21
G17

; Work coordinate at impeller center, top of hub
G0 A0 B0          ; Initialize rotary axes
T1 M6             ; Ball end mill 6mm
G43 H1
S12000 M3

#<center_x> = 0
#<center_y> = 0
#<safe_z> = 50

G0 Z#<safe_z>

; =============================================
; BLADE SURFACE POINT SUBROUTINE
; =============================================
; Calculates a point on twisted blade surface
; #1 = radial position (0=hub, 1=tip)
; #2 = span position (0=leading, 1=trailing edge)
; #3 = blade number
; Returns: #<bx>, #<by>, #<bz>, #<ba>, #<bb>
o100 sub (Blade Surface Point)
  ; Radial distance
  #<rad_pos> = #1
  #<span_pos> = #2
  #<blade_num> = #3
  
  #<radius> = [#<hub_dia>/2 + [#<outer_dia>/2 - #<hub_dia>/2] * #<rad_pos>]
  
  ; Height varies with radius (parabolic hub)
  #<z_height> = [-#<blade_height> * #<rad_pos> * #<rad_pos>]
  
  ; Blade angle - interpolate from inlet to outlet
  #<blade_angle> = [#<inlet_angle> + [#<outlet_angle> - #<inlet_angle>] * #<span_pos>]
  
  ; Add twist based on radial position
  #<twist> = [#<blade_twist> * #<rad_pos>]
  #<total_angle> = [#<blade_angle> + #<twist>]
  
  ; Rotation for blade number
  #<blade_rotation> = [[360 / #<num_blades>] * #<blade_num>]
  
  ; Calculate XY position
  #<local_x> = [#<radius> * COS[#<total_angle>]]
  #<local_y> = [#<radius> * SIN[#<total_angle>]]
  
  ; Rotate by blade position
  #<bx> = [#<local_x> * COS[#<blade_rotation>] - #<local_y> * SIN[#<blade_rotation>]]
  #<by> = [#<local_x> * SIN[#<blade_rotation>] + #<local_y> * COS[#<blade_rotation>]]
  #<bz> = #<z_height>
  
  ; Calculate tool orientation (A, B rotary)
  ; Normal to surface approximation
  #<ba> = [#<total_angle> * 0.5]   ; A-axis tilt
  #<bb> = [#<blade_rotation>]      ; B-axis rotation
o100 endsub

; =============================================
; CUT SINGLE BLADE
; =============================================
o200 sub (Machine One Blade)
  #<blade> = #1
  (MSG, Machining blade #<blade>)
  
  ; Roughing passes - radial strips
  #<r_step> = 0.1   ; 10% radial steps
  #<s_step> = 0.05  ; 5% span steps
  
  #<r_pos> = 0
  o210 while [#<r_pos> LE 1.0]
    
    ; Zigzag along span
    #<s_pos> = 0
    #<direction> = 1
    
    o220 while [#<s_pos> LE 1.0]
      ; Get surface point
      o100 call [#<r_pos>] [#<s_pos>] [#<blade>]
      
      ; Move with 5-axis orientation
      G1 X#<bx> Y#<by> Z#<bz> A#<ba> B#<bb> F600
      
      #<s_pos> = [#<s_pos> + #<s_step>]
    o220 endwhile
    
    ; Step radially
    #<r_pos> = [#<r_pos> + #<r_step>]
    
    ; Quick retract between strips
    G0 Z[#<bz> + 3]
  o210 endwhile
  
  G0 Z#<safe_z>
o200 endsub

; =============================================
; ROUGH HUB SURFACE
; =============================================
(MSG, Roughing hub surface)

; Spiral from center outward
#<spiral_z> = -2
#<spiral_r> = 5

G0 X0 Y0 Z#<safe_z>
G0 X#<spiral_r> Y0 A0 B0
G1 Z#<spiral_z> F300

o300 while [#<spiral_r> LT [#<hub_dia>/2]]
  ; Spiral outward
  G3 X#<spiral_r> Y0 I[-#<spiral_r>] J0 F800
  #<spiral_r> = [#<spiral_r> + 2]
  G1 X#<spiral_r> F800
o300 endwhile

G0 Z#<safe_z>

; =============================================
; MACHINE ALL BLADES
; =============================================
#<current_blade> = 0
o400 while [#<current_blade> LT #<num_blades>]
  o200 call [#<current_blade>]
  #<current_blade> = [#<current_blade> + 1]
o400 endwhile

; =============================================
; FINISH PASS - FLOW SURFACES
; =============================================
(MSG, Finish pass - flow surfaces)

T2 M6             ; 3mm ball end mill
G43 H2
S15000 M3

; Higher resolution finish
#<finish_r_step> = 0.02
#<finish_s_step> = 0.02

#<blade> = 0
o500 while [#<blade> LT #<num_blades>]
  
  #<r_pos> = 0
  o510 while [#<r_pos> LE 1.0]
    #<s_pos> = 0
    o520 while [#<s_pos> LE 1.0]
      o100 call [#<r_pos>] [#<s_pos>] [#<blade>]
      G1 X#<bx> Y#<by> Z#<bz> A#<ba> B#<bb> F400
      #<s_pos> = [#<s_pos> + #<finish_s_step>]
    o520 endwhile
    
    #<r_pos> = [#<r_pos> + #<finish_r_step>]
    G0 Z[#<bz> + 2]
  o510 endwhile
  
  #<blade> = [#<blade> + 1]
o500 endwhile

; =============================================
; END
; =============================================
G0 Z#<safe_z>
G0 A0 B0          ; Return rotary axes
G0 X0 Y50
M5
M30
%
