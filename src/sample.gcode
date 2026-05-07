; Sample GCode for GCodeFlow
; A simple test pattern to demonstrate visualization

; Home and set absolute positioning
G28 ; Home all axes
G90 ; Absolute positioning
M82 ; Absolute extruder mode

; Set feed rates
G1 F1500 ; Set default feed rate

; Move to start position
G0 Z5 ; Raise Z
G0 X10 Y10 ; Move to corner
G0 Z0.2 ; Lower to print height

; Draw a square
G1 X100 Y10 F1200 ; Bottom edge
G1 X100 Y100 ; Right edge
G1 X10 Y100 ; Top edge
G1 X10 Y10 ; Left edge

; Draw diagonal
G1 X100 Y100 ; Diagonal

; Draw a circle (approximated with arcs)
G0 X60 Y10 ; Move to circle start
G2 X60 Y10 I-10 J0 F800 ; Full circle (radius 10)

; Draw a larger circle
G0 X110 Y55 ; Move to circle start
G2 X110 Y55 I-25 J0 F600 ; Full circle (radius 25)

; Retract and move to center
G1 Z5 F1000 ; Raise
G0 X55 Y55 ; Move to center

; Draw a star pattern
G1 Z0.2 F1000
G1 X55 Y80 F1500
G1 X70 Y30
G1 X30 Y60
G1 X80 Y60
G1 X40 Y30
G1 X55 Y80

; Move up layers
G1 Z1 F600
G0 X20 Y20
G1 Z0.4
G1 X80 Y20 F1200
G1 X80 Y80
G1 X20 Y80
G1 X20 Y20

; Rapid move to home
G0 Z10 ; Raise
G0 X0 Y0 ; Return to origin

; End
M84 ; Disable motors
