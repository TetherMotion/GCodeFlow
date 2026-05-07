; Example 440: Adaptive Clearing with Trochoidal Milling
; High-efficiency material removal strategy

%
O4400 (ADAPTIVE TROCHOIDAL CLEARING)

; =============================================
; CONFIGURATION
; =============================================
#<tool_dia> = 10
#<tool_rad> = [#<tool_dia> / 2]
#<stepover> = [#<tool_dia> * 0.1]   ; 10% stepover for trochoidal
#<loop_radius> = [#<tool_dia> * 0.4] ; Trochoidal loop radius
#<feed_rate> = 2000                   ; High feed for trochoidal
#<plunge_feed> = 500
#<depth_step> = 3

; Pocket definition
#<pocket_x> = 20
#<pocket_y> = 20  
#<pocket_w> = 100
#<pocket_h> = 80
#<pocket_depth> = -15

; =============================================
; SETUP
; =============================================
G90 G54
G21
G17

T1 M6
G43 H1
S8000 M3
M8                ; Coolant on

G0 Z25
G0 X[#<pocket_x> + #<tool_rad> + #<loop_radius>] Y[#<pocket_y> + #<tool_rad>]

; =============================================
; TROCHOIDAL MOVE SUBROUTINE
; =============================================
; Moves from current position toward target using trochoidal path
; #1 = target X, #2 = target Y, #3 = Z depth
o100 sub (Trochoidal Move)
  #<targ_x> = #1
  #<targ_y> = #2
  #<targ_z> = #3
  
  ; Get current position (use starting position)
  #<cur_x> = #5001
  #<cur_y> = #5002
  
  ; Calculate direction vector
  #<dx> = [#<targ_x> - #<cur_x>]
  #<dy> = [#<targ_y> - #<cur_y>]
  #<dist> = SQRT[[#<dx>*#<dx>] + [#<dy>*#<dy>]]
  
  o110 if [#<dist> GT 0.1]
    ; Normalize direction
    #<nx> = [#<dx> / #<dist>]
    #<ny> = [#<dy> / #<dist>]
    
    ; Perpendicular vector for loops
    #<px> = [-#<ny>]
    #<py> = [#<nx>]
    
    ; Number of loops
    #<num_loops> = FIX[[#<dist> / #<stepover>]]
    o115 if [#<num_loops> LT 1]
      #<num_loops> = 1
    o115 endif
    
    ; Step size per loop
    #<step> = [#<dist> / #<num_loops>]
    
    ; Generate trochoidal path
    #<i> = 0
    o120 while [#<i> LT #<num_loops>]
      ; Center of this loop
      #<cx> = [#<cur_x> + #<nx> * #<step> * [#<i> + 0.5]]
      #<cy> = [#<cur_y> + #<ny> * #<step> * [#<i> + 0.5]]
      
      ; Arc entry (climb milling)
      ; Start arc from approaching side
      #<arc_start_x> = [#<cx> - #<px> * #<loop_radius>]
      #<arc_start_y> = [#<cy> - #<py> * #<loop_radius>]
      
      G1 X#<arc_start_x> Y#<arc_start_y> F[#<feed_rate>]
      
      ; Full circle trochoidal loop
      G3 X#<arc_start_x> Y#<arc_start_y> I[#<px> * #<loop_radius>] J[#<py> * #<loop_radius>] F[#<feed_rate>]
      
      #<i> = [#<i> + 1]
    o120 endwhile
    
    ; Move to target
    G1 X#<targ_x> Y#<targ_y> F[#<feed_rate>]
  o110 endif
o100 endsub

; =============================================
; ADAPTIVE POCKETING WITH DEPTH STEPPING
; =============================================
#<current_z> = 0

o200 while [#<current_z> GT #<pocket_depth>]
  #<current_z> = [#<current_z> - #<depth_step>]
  o210 if [#<current_z> LT #<pocket_depth>]
    #<current_z> = #<pocket_depth>
  o210 endif
  
  (MSG, Clearing at Z #<current_z>)
  
  ; Plunge to depth with helix
  G0 X[#<pocket_x> + #<tool_rad> * 2] Y[#<pocket_y> + #<tool_rad> * 2]
  G0 Z[#<current_z> + #<depth_step> + 2]
  
  ; Helical entry
  #<helix_rad> = [#<tool_rad> * 0.8]
  #<helix_start_z> = [#<current_z> + #<depth_step>]
  o220 while [#<helix_start_z> GT #<current_z>]
    G3 I#<helix_rad> J0 Z[#<helix_start_z> - 0.5] F#<plunge_feed>
    #<helix_start_z> = [#<helix_start_z> - 0.5]
  o220 endwhile
  
  ; Clear pocket row by row with trochoidal paths
  #<row_y> = [#<pocket_y> + #<tool_rad>]
  #<row_step> = [#<loop_radius> * 2 + #<stepover>]
  
  o230 while [#<row_y> LT [#<pocket_y> + #<pocket_h> - #<tool_rad>]]
    ; Trochoidal pass left to right
    G1 X[#<pocket_x> + #<tool_rad>] Y#<row_y> F#<feed_rate>
    o100 call [#<pocket_x> + #<pocket_w> - #<tool_rad>] [#<row_y>] [#<current_z>]
    
    #<row_y> = [#<row_y> + #<row_step>]
    
    o235 if [#<row_y> LT [#<pocket_y> + #<pocket_h> - #<tool_rad>]]
      ; Move to next row
      G1 Y#<row_y> F#<feed_rate>
      ; Trochoidal pass right to left  
      o100 call [#<pocket_x> + #<tool_rad>] [#<row_y>] [#<current_z>]
      #<row_y> = [#<row_y> + #<row_step>]
    o235 endif
  o230 endwhile
  
  G0 Z[#<current_z> + 2]
o200 endwhile

; =============================================
; FINISH PASS - CONVENTIONAL CONTOUR
; =============================================
(MSG, Finish pass)

G0 X[#<pocket_x> + #<tool_rad>] Y[#<pocket_y> + #<tool_rad>]
G1 Z#<pocket_depth> F#<plunge_feed>

; Climb milling contour
G1 X[#<pocket_x> + #<pocket_w> - #<tool_rad>] F800
G1 Y[#<pocket_y> + #<pocket_h> - #<tool_rad>]
G1 X[#<pocket_x> + #<tool_rad>]
G1 Y[#<pocket_y> + #<tool_rad>]

G0 Z25

; =============================================
; CLEANUP
; =============================================
G0 X0 Y0
M9                ; Coolant off
M5                ; Spindle stop
M30
%
