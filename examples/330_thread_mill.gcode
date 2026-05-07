; Example 330: Thread Milling
; Internal and external thread milling patterns

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

G0 Z20            ; Safe height

; =============================================
; Thread Milling Parameters
; =============================================
; For M20x2.5 thread:
#<major_dia> = 20
#<pitch> = 2.5
#<thread_depth> = 1.35  ; Standard metric depth
#<tool_dia> = 8         ; Thread mill diameter

; =============================================
; Internal Thread (in a pre-drilled hole)
; Location: (40, 40)
; =============================================

#<hole_center_x> = 40
#<hole_center_y> = 40
#<thread_length> = 15

; Calculate helix radius for internal thread
#<int_helix_r> = [[#<major_dia> - #<tool_dia>] / 2 - #<thread_depth>]

; Position above hole
G0 X[#<hole_center_x> + #<int_helix_r>] Y[#<hole_center_y>]
G0 Z5

; Helical entry to thread start depth
G1 Z0 F200

; Thread milling - CCW for RH internal thread
; Each revolution descends by pitch
#<thread_z> = 0
#<passes> = [#<thread_length> / #<pitch>]
#<pass> = 0

o100 while [#<pass> LT #<passes>]
  #<thread_z> = [#<thread_z> - #<pitch>]
  G3 X[#<hole_center_x> + #<int_helix_r>] Y[#<hole_center_y>] Z[#<thread_z>] I[-#<int_helix_r>] J0 F150
  #<pass> = [#<pass> + 1]
o100 endwhile

; Exit move
G0 X[#<hole_center_x>] Y[#<hole_center_y>]
G0 Z10

; =============================================
; External Thread (on a post/boss)
; Location: (110, 40)
; =============================================

#<boss_center_x> = 110
#<boss_center_y> = 40
#<ext_thread_length> = 12

; Calculate helix radius for external thread  
#<ext_helix_r> = [[#<major_dia> + #<tool_dia>] / 2 + #<thread_depth>]

; Position above boss
G0 X[#<boss_center_x> + #<ext_helix_r>] Y[#<boss_center_y>]
G0 Z5
G1 Z0 F200

; Thread milling - CW for RH external thread
#<thread_z> = 0
#<passes> = [#<ext_thread_length> / #<pitch>]
#<pass> = 0

o200 while [#<pass> LT #<passes>]
  #<thread_z> = [#<thread_z> - #<pitch>]
  G2 X[#<boss_center_x> + #<ext_helix_r>] Y[#<boss_center_y>] Z[#<thread_z>] I[-#<ext_helix_r>] J0 F150
  #<pass> = [#<pass> + 1]
o200 endwhile

G0 Z10

; =============================================
; Tapered Thread (NPT style)
; Location: (75, 110)
; Taper: 1:16 (about 1.79 degrees)
; =============================================

#<taper_center_x> = 75
#<taper_center_y> = 110
#<taper_start_dia> = 22
#<taper_rate> = [1/16]  ; Diameter reduction per unit length
#<taper_pitch> = 2.0
#<taper_length> = 12

; Start at larger diameter
#<current_dia> = #<taper_start_dia>
#<taper_helix_r> = [[#<current_dia> - #<tool_dia>] / 2 - #<thread_depth>]

G0 X[#<taper_center_x> + #<taper_helix_r>] Y[#<taper_center_y>]
G0 Z5
G1 Z0 F200

#<thread_z> = 0
#<passes> = [#<taper_length> / #<taper_pitch>]
#<pass> = 0

o300 while [#<pass> LT #<passes>]
  #<thread_z> = [#<thread_z> - #<taper_pitch>]
  
  ; Calculate new diameter at this depth (tapering smaller)
  #<current_dia> = [#<taper_start_dia> - ABS[#<thread_z>] * #<taper_rate> * 2]
  #<taper_helix_r> = [[#<current_dia> - #<tool_dia>] / 2 - #<thread_depth>]
  
  ; Helical move with changing radius
  G3 X[#<taper_center_x> + #<taper_helix_r>] Y[#<taper_center_y>] Z[#<thread_z>] I[-#<taper_helix_r>] J0 F150
  
  #<pass> = [#<pass> + 1]
o300 endwhile

G0 Z20            ; Retract
G0 X0 Y0          ; Return home
M30
