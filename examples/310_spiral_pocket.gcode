; Example 310: Spiral Pocket Clearing
; Demonstrates spiral toolpath for circular pocket

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

G0 Z10            ; Safe height

; =============================================
; Spiral Pocket Parameters
; =============================================
#<center_x> = 75
#<center_y> = 75
#<pocket_radius> = 40
#<tool_radius> = 3
#<stepover> = [#<tool_radius> * 1.5]  ; 75% stepover
#<depth> = -15
#<depth_step> = 5

; =============================================
; Spiral Pocket Subroutine
; Cuts from center outward in expanding spiral
; Parameters: #1=depth for this pass
; =============================================
o100 sub
  ; Start at center
  G0 X[#<center_x>] Y[#<center_y>]
  G1 Z[#1] F100
  
  ; Spiral outward
  #<current_r> = [#<stepover>]
  
  o110 while [#<current_r> LT #<pocket_radius> - #<tool_radius>]
    ; One complete revolution at this radius
    ; We'll do it in 4 quarter arcs for smoother spiral
    
    #<next_r> = [#<current_r> + #<stepover> / 4]
    o120 if [#<next_r> GT #<pocket_radius> - #<tool_radius>]
      #<next_r> = [#<pocket_radius> - #<tool_radius>]
    o120 endif
    
    ; Quarter 1: 0 to 90 degrees
    #<x1> = [#<center_x> + #<current_r>]
    #<y1> = [#<center_y>]
    #<x2> = [#<center_x>]
    #<y2> = [#<center_y> + #<next_r>]
    
    G0 X[#<x1>] Y[#<y1>]
    G3 X[#<x2>] Y[#<y2>] I[-#<current_r>] J0 F400
    
    #<current_r> = [#<current_r> + #<stepover> / 4]
    o121 if [#<current_r> GT #<pocket_radius> - #<tool_radius>]
      #<current_r> = [#<pocket_radius> - #<tool_radius>]
    o121 endif
    
    ; Quarter 2: 90 to 180 degrees
    #<x3> = [#<center_x> - #<current_r>]
    #<y3> = [#<center_y>]
    G3 X[#<x3>] Y[#<y3>] I0 J[-#<current_r>] F400
    
    #<current_r> = [#<current_r> + #<stepover> / 4]
    o122 if [#<current_r> GT #<pocket_radius> - #<tool_radius>]
      #<current_r> = [#<pocket_radius> - #<tool_radius>]
    o122 endif
    
    ; Quarter 3: 180 to 270 degrees
    #<x4> = [#<center_x>]
    #<y4> = [#<center_y> - #<current_r>]
    G3 X[#<x4>] Y[#<y4>] I[#<current_r>] J0 F400
    
    #<current_r> = [#<current_r> + #<stepover> / 4]
    o123 if [#<current_r> GT #<pocket_radius> - #<tool_radius>]
      #<current_r> = [#<pocket_radius> - #<tool_radius>]
    o123 endif
    
    ; Quarter 4: 270 to 360 degrees
    #<x5> = [#<center_x> + #<current_r>]
    #<y5> = [#<center_y>]
    G3 X[#<x5>] Y[#<y5>] I0 J[#<current_r>] F400
    
  o110 endwhile
  
  ; Final cleanup pass at full radius
  #<final_r> = [#<pocket_radius> - #<tool_radius>]
  G0 X[#<center_x> + #<final_r>] Y[#<center_y>]
  G3 I[-#<final_r>] J0 F400   ; Full circle
  
  G0 Z[#1 + 3]    ; Small retract
o100 endsub

; =============================================
; Main Program - Multiple depth passes
; =============================================

#<current_depth> = 0
o200 while [#<current_depth> GT #<depth>]
  #<current_depth> = [#<current_depth> - #<depth_step>]
  o210 if [#<current_depth> LT #<depth>]
    #<current_depth> = #<depth>
  o210 endif
  
  (MSG, Cutting at depth #<current_depth>)
  o100 call [#<current_depth>]
o200 endwhile

G0 Z10            ; Retract
G0 X0 Y0          ; Return home
M30
