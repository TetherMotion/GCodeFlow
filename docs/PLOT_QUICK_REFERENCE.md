# 2D Plot Analysis - Quick Reference

## Opening the Plot

**Menu**: View → 📊 2D Plot...

## Plot Modes

| Mode | Purpose | Use Case |
|------|---------|----------|
| **Time-based** | View parameters vs time | Analyze velocity profiles, acceleration limits |
| **2D Projection** | View XY/XZ/YZ path | Verify geometric accuracy, corner rounding |

## Time-Based Controls

**Select Curves**: 
- ☑ Position (mm) - *default ON*
- ☑ Velocity (mm/s) - *default ON*
- ☐ Acceleration (mm/s²)
- ☐ Jerk (mm/s³)

**Select Axis**: X | Y | Z | E

## Cartesian Controls

**Horizontal Axis**: X | Y | Z  
**Vertical Axis**: X | Y | Z  

*Default: X (horizontal) × Y (vertical)*

## Mouse Controls

| Action | Control |
|--------|---------|
| **Pan** | Left-click + drag |
| **Zoom** | Mouse wheel |
| **Select Point** | Left-click on curve |
| **Reset View** | Enable Auto-fit |

## Point Detail Popup

Click any point on the Position curve to see:
- Time (s)
- Position - all axes (mm)
- Velocity - all axes (mm/s) + magnitude
- Acceleration - all axes (mm/s²) + magnitude
- GCode block index

## Tips

✓ Use **Time-based** mode to check if velocity limits are respected  
✓ Use **2D Projection** to verify corner paths match your intent  
✓ Enable **Auto-fit** when switching between GCode segments  
✓ Disable unused curves to improve performance on large files  
✓ Check **Acceleration** curve for smooth transitions (no spikes)  

## Keyboard Shortcuts

*Planned - not yet implemented*

## Troubleshooting

**Empty Plot?**
- Ensure GCode file is loaded
- Check selected axis has motion
- Verify trajectory generation succeeded

**Slow Performance?**
- Disable Jerk curve if not needed
- Zoom into smaller time ranges
- Consider reducing GCode complexity
