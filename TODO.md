# Things with no dependencies
FM Oscillator
minifying preset files
Version number on preset files so that new versions of the format can be made
B64-encode of preset files
LFO
Transposer
Beat sequencer
Pitch sequencer
Center the canvas when loading a new preset
scroll to zoom

# Things with missing dependencies

# Low-priority things without dependencies
Int boxes are a pain to use because double-click.
Output silent audio while recompiling instead of hanging the thread
Get a better icon for waveforms
Better nothing icon
Highlight connections when hovering over things like outputs and automation
  lanes to see more easily what's connected to what
Play a pretend note when moving knobs so you can see its effect without playing your own note.
Undo / redo
Make the add modules menu look nice
Make window resizable

# Code organization stuff
Use more TupleUtil functions
ModuleLibrary -> ModuleCatalog
Control -> Parameter?
Parameter -> AutoParam, ComplexParam?
use fewer i32s, replace with usizes when it would be helpful.
Tidy up warnings

# Long-term goals
Effects graph
MIDI graph
Preset tags
Some kind of custom GUI creation or custom module creation (without programming knowledge)
Builtin modules for complex flow / codegen stuff 
  Auto stack / chain modules
  Option module
Undo / redo tree
Library catalog

# Forward compatibility things
Presets should have spots for multiple graphs
Presets should store tags, just a list of strings
Builtin modules can use the same save syntax as other modules, using one or more complex controls to
  store their extra data.
Some kind of library identification file with information like the minimum version of audiobench you
  need to use the library
