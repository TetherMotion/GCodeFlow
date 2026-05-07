; Example 430: Complex Geometry - Involute Gear
; Calculates and cuts an involute gear profile

%
O4300 (INVOLUTE GEAR CUTTING)

; =============================================
; GEAR PARAMETERS
; =============================================
; Module 2 gear with 24 teeth

#<module> = 2
#<num_teeth> = 24
#<pressure_angle> = 20    ; degrees

; Calculated values
#<pitch_dia> = [#<module> * #<num_teeth>]      ; 48mm
#<base_dia> = [#<pitch_dia> * COS[#<pressure_angle>]]
#<outer_dia> = [#<pitch_dia> + 2 * #<module>]  ; 52mm
#<root_dia> = [#<pitch_dia> - 2.5 * #<module>] ; 43mm

#<tooth_angle> = [360 / #<num_teeth>]          ; 15 degrees per tooth
#<gear_depth> = 5

(MSG, Gear: M#<module> Z#<num_teeth>)
(MSG, Pitch Diameter: #<pitch_dia>)
(MSG, Outer Diameter: #<outer_dia>)

; =============================================
; SETUP
; =============================================
G90 G54
G21
G17

T1 M6             ; 1mm end mill for gear profile
G43 H1
S10000 M3

#<center_x> = 75
#<center_y> = 75

G0 Z25
G0 X#<center_x> Y#<center_y>

; =============================================
; INVOLUTE POINT CALCULATION SUBROUTINE
; =============================================
; Returns X, Y for involute curve at angle
; #1 = involute angle (radians)
; Output: #<inv_x>, #<inv_y>
o100 sub (Calculate Involute Point)
  #<base_r> = [#<base_dia> / 2]
  
  ; Involute: x = r*(cos(t) + t*sin(t))
  ;           y = r*(sin(t) - t*cos(t))
  #<inv_x> = [#<base_r> * [COS[#1] + #1 * SIN[#1]]]
  #<inv_y> = [#<base_r> * [SIN[#1] - #1 * COS[#1]]]
o100 endsub

; =============================================
; SINGLE TOOTH PROFILE SUBROUTINE
; =============================================
o200 sub (Cut One Tooth Profile)
  ; #1 = tooth number (0 to num_teeth-1)
  #<tooth_rotation> = [#1 * #<tooth_angle>]
  
  ; Start at root circle
  #<start_angle> = #<tooth_rotation>
  #<root_r> = [#<root_dia> / 2]
  
  #<px> = [#<center_x> + #<root_r> * COS[#<start_angle>]]
  #<py> = [#<center_y> + #<root_r> * SIN[#<start_angle>]]
  
  G0 X#<px> Y#<py>
  G1 Z[-#<gear_depth>] F100
  
  ; Trace involute curve - right flank
  #<inv_angle> = 0
  #<outer_r> = [#<outer_dia> / 2]
  
  o210 while [#<inv_angle> LT 0.6]
    o100 call [#<inv_angle>]
    
    ; Rotate involute point by tooth angle
    #<dist> = SQRT[[#<inv_x>*#<inv_x>] + [#<inv_y>*#<inv_y>]]
    #<ang> = ATAN[#<inv_y>] / [#<inv_x>]
    #<ang> = [#<ang> + #<tooth_rotation>]
    
    ; Only cut if within outer diameter
    o215 if [#<dist> LT #<outer_r>]
      #<px> = [#<center_x> + #<dist> * COS[#<ang>]]
      #<py> = [#<center_y> + #<dist> * SIN[#<ang>]]
      G1 X#<px> Y#<py> F300
    o215 endif
    
    #<inv_angle> = [#<inv_angle> + 0.05]
  o210 endwhile
  
  ; Arc across tooth tip
  #<tip_angle> = [#<tooth_rotation> + #<tooth_angle>/4]
  #<px> = [#<center_x> + #<outer_r> * COS[#<tip_angle>]]
  #<py> = [#<center_y> + #<outer_r> * SIN[#<tip_angle>]]
  G3 X#<px> Y#<py> I[#<center_x> - #5001] J[#<center_y> - #5002] ; Arc to tip
  
  ; Return down left flank (mirror of involute)
  #<inv_angle> = 0.6
  o220 while [#<inv_angle> GT 0]
    o100 call [#<inv_angle>]
    
    ; Mirror and rotate
    #<dist> = SQRT[[#<inv_x>*#<inv_x>] + [#<inv_y>*#<inv_y>]]
    #<ang> = ATAN[#<inv_y>] / [#<inv_x>]
    #<ang> = [[#<tooth_rotation> + #<tooth_angle>/2] - #<ang>]  ; Mirror
    
    o225 if [#<dist> LT #<outer_r>]
      #<px> = [#<center_x> + #<dist> * COS[#<ang>]]
      #<py> = [#<center_y> + #<dist> * SIN[#<ang>]]
      G1 X#<px> Y#<py> F300
    o225 endif
    
    #<inv_angle> = [#<inv_angle> - 0.05]
  o220 endwhile
  
  ; Back to root
  #<end_angle> = [#<tooth_rotation> + #<tooth_angle>/2]
  #<px> = [#<center_x> + #<root_r> * COS[#<end_angle>]]
  #<py> = [#<center_y> + #<root_r> * SIN[#<end_angle>]]
  G1 X#<px> Y#<py> F300
  
  G0 Z2
o200 endsub

; =============================================
; MAIN CUTTING LOOP
; =============================================
(MSG, Cutting #<num_teeth> teeth)

#<tooth> = 0
o300 while [#<tooth> LT #<num_teeth>]
  (MSG, Tooth #<tooth>)
  o200 call [#<tooth>]
  #<tooth> = [#<tooth> + 1]
o300 endwhile

; =============================================
; CUT CENTER BORE
; =============================================
(MSG, Cutting center bore)

T2 M6             ; 4mm end mill
G43 H2
S8000 M3

#<bore_dia> = 10
G0 X#<center_x> Y[#<center_y> + #<bore_dia>/2 - 2]
G1 Z[-#<gear_depth>] F200

; Helical bore
#<z> = 0
o400 while [#<z> GT [-#<gear_depth>]]
  G3 I0 J[-[#<bore_dia>/2 - 2]] Z[#<z> - 0.5] F400
  #<z> = [#<z> - 0.5]
o400 endwhile

; Final pass
G3 I0 J[-[#<bore_dia>/2 - 2]] F400
G3 I0 J[-[#<bore_dia>/2 - 2]] F400

G0 Z25

; =============================================
; FINISH
; =============================================
G0 X0 Y0 Z50
M5
M30
%
