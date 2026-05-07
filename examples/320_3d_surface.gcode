; Example 320: 3D Surface Machining
; Demonstrates Z-varying toolpaths for 3D shapes

G90               ; Absolute mode
G21               ; Metric
G17               ; XY plane

G0 Z20            ; Safe height

; =============================================
; 3D Hemisphere Surface
; Creates a dome shape using calculated Z heights
; =============================================

#<center_x> = 50
#<center_y> = 50
#<radius> = 40
#<stepover> = 2

; Raster pattern across the surface
#<y> = [#<center_y> - #<radius>]

o100 while [#<y> LE [#<center_y> + #<radius>]]
  
  ; Calculate chord width at this Y
  #<dy> = [#<y> - #<center_y>]
  #<chord_half> = [SQRT[#<radius> * #<radius> - #<dy> * #<dy>]]
  
  o110 if [#<chord_half> GT 0]
    ; Scan line from left to right
    #<x_start> = [#<center_x> - #<chord_half>]
    #<x_end> = [#<center_x> + #<chord_half>]
    
    ; Move to start of line
    G0 X[#<x_start>] Y[#<y>]
    
    ; Calculate Z at start (on the sphere surface)
    #<dx> = [#<x_start> - #<center_x>]
    #<z_start> = [SQRT[#<radius> * #<radius> - #<dx> * #<dx> - #<dy> * #<dy>]]
    G1 Z[-#<radius> + #<z_start>] F200
    
    ; Traverse across with varying Z
    #<x> = [#<x_start>]
    o120 while [#<x> LE #<x_end>]
      #<dx> = [#<x> - #<center_x>]
      #<dist_sq> = [#<dx> * #<dx> + #<dy> * #<dy>]
      
      o130 if [#<dist_sq> LE [#<radius> * #<radius>]]
        #<z_val> = [SQRT[#<radius> * #<radius> - #<dist_sq>]]
        G1 X[#<x>] Z[-#<radius> + #<z_val>] F300
      o130 endif
      
      #<x> = [#<x> + 2]  ; Step in X
    o120 endwhile
    
    G0 Z5
  o110 endif
  
  #<y> = [#<y> + #<stepover>]
o100 endwhile

; =============================================
; 3D Wave Surface
; Sinusoidal height variation
; =============================================

#<wave_x_start> = 110
#<wave_y_start> = 10
#<wave_width> = 60
#<wave_length> = 60
#<amplitude> = 8
#<wavelength> = 15

G0 Z10
G0 X[#<wave_x_start>] Y[#<wave_y_start>]

#<wy> = 0
o200 while [#<wy> LE #<wave_length>]
  #<wx> = 0
  G0 X[#<wave_x_start>] Y[#<wave_y_start> + #<wy>]
  
  o210 while [#<wx> LE #<wave_width>]
    ; Calculate wave height: sin(x) * sin(y)
    #<z_wave> = [#<amplitude> * SIN[#<wx> * 360 / #<wavelength>] * SIN[#<wy> * 360 / #<wavelength>]]
    
    G1 X[#<wave_x_start> + #<wx>] Z[-5 + #<z_wave>] F400
    
    #<wx> = [#<wx> + 1]
  o210 endwhile
  
  G0 Z5
  #<wy> = [#<wy> + 2]
o200 endwhile

; =============================================
; 3D Conical Ramp
; Linear height change in spiral
; =============================================

#<cone_x> = 50
#<cone_y> = 130
#<cone_base_r> = 30
#<cone_height> = 20
#<spiral_step> = 3

G0 Z10
G0 X[#<cone_x> + #<cone_base_r>] Y[#<cone_y>]

#<r> = #<cone_base_r>
o300 while [#<r> GT 2]
  ; Calculate Z based on radius (cone shape)
  #<z_cone> = [#<cone_height> * [1 - #<r> / #<cone_base_r>]]
  
  ; Spiral inward
  G2 X[#<cone_x> + #<r> - #<spiral_step>] Y[#<cone_y>] Z[-#<z_cone>] I[-#<r>] J0 F300
  
  #<r> = [#<r> - #<spiral_step>]
o300 endwhile

G0 Z20            ; Retract
G0 X0 Y0          ; Return home
M30
