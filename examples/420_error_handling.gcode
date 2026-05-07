; Example 420: Error Handling and Recovery
; Demonstrates conditional logic for error scenarios

%
O4200 (ERROR HANDLING EXAMPLE)

; =============================================
; CONFIGURATION WITH VALIDATION
; =============================================

; Input parameters
#<tool_diameter> = 10
#<depth_of_cut> = 3
#<material_hardness> = 150   ; BHN
#<max_feed> = 1500
#<max_rpm> = 10000

; Calculated limits
#<min_tool_dia> = 3
#<max_depth_ratio> = 1.5     ; Max depth = 1.5 * tool diameter

; =============================================
; PARAMETER VALIDATION SUBROUTINE
; =============================================
o100 sub (Validate Parameters)
  #<valid> = 1
  
  ; Check tool diameter
  o110 if [#<tool_diameter> LT #<min_tool_dia>]
    (MSG, ERROR: Tool diameter too small)
    #<valid> = 0
  o110 endif
  
  ; Check depth of cut ratio
  #<max_doc> = [#<tool_diameter> * #<max_depth_ratio>]
  o120 if [#<depth_of_cut> GT #<max_doc>]
    (MSG, WARNING: Depth of cut exceeds recommendation)
    (MSG, Max recommended: #<max_doc>)
    ; Auto-correct
    #<depth_of_cut> = #<max_doc>
    (MSG, Adjusted to: #<depth_of_cut>)
  o120 endif
  
  ; Calculate recommended feed based on material
  o130 if [#<material_hardness> LT 100]
    ; Soft material - aluminum
    #<rec_feed> = 1200
    #<rec_rpm> = 8000
  o130 elseif [#<material_hardness> LT 200]
    ; Medium material - mild steel
    #<rec_feed> = 600
    #<rec_rpm> = 4000
  o130 elseif [#<material_hardness> LT 300]
    ; Hard material - stainless
    #<rec_feed> = 300
    #<rec_rpm> = 2500
  o130 else
    ; Very hard - limit parameters
    #<rec_feed> = 150
    #<rec_rpm> = 1500
    (MSG, WARNING: Very hard material - reduced parameters)
  o130 endif
  
  ; Apply limits
  o140 if [#<rec_feed> GT #<max_feed>]
    #<rec_feed> = #<max_feed>
  o140 endif
  
  o150 if [#<rec_rpm> GT #<max_rpm>]
    #<rec_rpm> = #<max_rpm>
  o150 endif
  
o100 endsub

; =============================================
; SAFE MOVEMENT SUBROUTINE
; =============================================
o200 sub (Safe Move - X, Y, Z)
  ; #1 = X target, #2 = Y target, #3 = Z target
  ; Always moves Z safe first, then XY, then Z down
  
  #<safe_z> = 25
  #<current_z> = #5063  ; System variable for Z position (simulated)
  
  ; If moving up, go directly
  o210 if [#3 GE #<current_z>]
    G0 Z#3
    G0 X#1 Y#2
  o210 else
    ; If moving down, safe retract first
    G0 Z#<safe_z>
    G0 X#1 Y#2
    G0 Z[#3 + 5]        ; Rapid to just above
    G1 Z#3 F200         ; Slow plunge
  o210 endif
o200 endsub

; =============================================
; COLLISION CHECK SUBROUTINE
; =============================================
o300 sub (Check Fixture Clearance)
  ; #1 = X, #2 = Y
  ; Define fixture boundaries
  #<fix_x1> = 50
  #<fix_y1> = 50
  #<fix_x2> = 100
  #<fix_y2> = 100
  
  #<clearance_ok> = 1
  
  ; Check if point is within fixture zone
  o310 if [#1 GT #<fix_x1>]
    o320 if [#1 LT #<fix_x2>]
      o330 if [#2 GT #<fix_y1>]
        o340 if [#2 LT #<fix_y2>]
          #<clearance_ok> = 0
          (MSG, ERROR: Point within fixture zone!)
        o340 endif
      o330 endif
    o320 endif
  o310 endif
o300 endsub

; =============================================
; TOOL BREAKAGE RECOVERY
; =============================================
o400 sub (Breakage Recovery)
  ; Simulated breakage detection response
  (MSG, ALERT: Tool breakage detected!)
  
  ; Emergency retract
  G0 Z50
  
  ; Spindle stop
  M5
  
  ; Coolant off
  M9
  
  ; Move to tool change position
  G0 X0 Y200
  
  ; Alert operator
  (MSG, Change tool and restart from safe point)
  
  ; Set recovery flag
  #<recovery_needed> = 1
o400 endsub

; =============================================
; MAIN PROGRAM
; =============================================
G90 G54
G21
G17

; Validate parameters
o100 call
(MSG, Feed: #<rec_feed>, RPM: #<rec_rpm>)

; Abort if invalid
o500 if [#<valid> EQ 0]
  (MSG, Cannot proceed - parameters invalid)
  M30
o500 endif

; Setup
T1 M6
G43 H1
S[#<rec_rpm>] M3

; Example operation with error checking
#<target_x> = 75
#<target_y> = 75
#<target_z> = -10

; Check fixture clearance
o300 call [#<target_x>] [#<target_y>]

o510 if [#<clearance_ok> EQ 1]
  ; Safe to proceed
  o200 call [#<target_x>] [#<target_y>] [#<target_z>]
  
  ; Cutting operation
  G1 X[#<target_x> + 50] F[#<rec_feed>]
  G1 Y[#<target_y> + 50]
  G1 X[#<target_x>]
  G1 Y[#<target_y>]
  
o510 else
  (MSG, Skipping operation due to clearance issue)
o510 endif

; Clean finish
G0 Z50
G0 X0 Y0
M5
M30
%
