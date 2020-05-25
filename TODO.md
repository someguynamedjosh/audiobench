# Things with no dependencies
Master volume control
Add option to draw white icons.
Categories for modules.
save and load presets
Make the add modules menu look nice
Some kind of Nodespeak library to simplify common tasks like updating waveform displays.
Friendly error messages
A widget to select from a series of options
Right-click menu should show automated value
Make arrangement of wires line up more with the grid system for controls.

# Things with missing dependencies
Make screen selector and int box use white icons
Options to change how automation channels are mixed together
FM Oscillator

# Low-priority things without dependencies
Output silent audio while recompiling instead of hanging the thread
Get a better icon for waveforms
Better nothing icon
Highlight connections when hovering over things like outputs and automation
  lanes to see more easily what's connected to what
scroll to zoom
Play a pretend note when moving knobs so you can see its effect without playing your own note.
Undo / redo

# Code organization stuff
Use more TupleUtil functions
ModuleLibrary -> ModuleCatalog
Control -> Parameter?
Parameter -> AutoParam, ComplexParam?
use fewer i32s, replace with usizes when it would be helpful.
Tidy up warnings
