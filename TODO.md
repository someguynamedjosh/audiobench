# Things with no dependencies
A way to remove automation channels and modules
Average together multiple automation channels (instead of add)
FN Oscillator
Master volume control
Add option to draw white icons.
Categories for modules.
save and load presets
Make the add modules menu look nice

# Things with missing dependencies
Make screen selector and int box use white icons

# Low-priority things without dependencies
Organize modules into categories (both GUI and folder structure wise)
Control suffixes.
Some kind of Nodespeak library to simplify common tasks like updating waveform displays.
Output silent audio while recompiling instead of hanging the thread
Display numeric value of control when manually dragging it
Get a better icon for waveforms
Better nothing icon
Friendly error messages
Options to change how automation channels are mixed together
Highlight connections when hovering over things like outputs and automation
  lanes to see more easily what's connected to what
scroll to zoom
Right-click menu should show automated value
Play a pretend note when moving knobs so you can see its effect without playing your own note.

# Code organization stuff
Use more TupleUtil functions
ModuleLibrary -> ModuleCatalog
Control -> Parameter?
Parameter -> AutoParam, ComplexParam?
use fewer i32s, replace with usizes when it would be helpful.
Tidy up warnings
