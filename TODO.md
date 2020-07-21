# Things with no dependencies
- Version checks so that new presets don't get loaded with old libraries
- More visual feedback for typing widget
- Clicking on the ends of channels with more than one channel picks the wrong channel
- Pitch wheel deadzone
- Mixer module
- Noise module
- Scrollbar for patch list
- Comb filter
- Loading the current state while having bad libraries causes a message "patch 
  data is corrupt". It only shows the correct error after resetting the current
  patch.
- Copy error reports to clipboard
- Convert all indexes[] in save_data.rs to checked get() calls
- Don't select a patch when clicking on it if loading it caused an error

# Things with missing dependencies
- Variable range for pitch wheel

# Low-priority things without dependencies
- Output silent audio while recompiling instead of hanging the thread
- Highlight connections when hovering over things like outputs and automation
  lanes to see more easily what's connected to what
- Play a pretend note when moving knobs so you can see its effect without playing your own note.
- Undo / redo
- Make the add modules menu look better
- Search filters for the add module menu
  - alphabetical sort vs category sort
  - require certain inputs / outputs
  - scrollbar too maybe
- Make window resizable
- Nicer error when a patch requires newer modules (not just "patch is corrupt")
- Higher precision for MIDI controls
- Optional smoothing for MIDI controls
- Reorganize engine modules and their contents
- Make website look nice
- Things for making waveforms using harmonics of other waveforms
- Update tooltip when clicking on timing control
- Add and/or/xor to nodespeak
- Optional variable smoothing for MIDI controls
- Line artifact when rendering default inputs
- Better integration with other VST library preset methods
- Dependency checks between libraries

# Code organization stuff
- Use more TupleUtil functions
- ModuleLibrary -> ModuleCatalog
- Control -> Parameter?
- Parameter -> AutoParam, ComplexParam?
- use fewer i32s, replace with usizes when it would be helpful.
- Tidy up warnings

# Long-term goals
- Effects graph
- MIDI graph
- patch tags
- Some kind of custom GUI creation or custom module creation (without programming knowledge)
- Builtin modules for complex flow / codegen stuff 
  - Auto stack / chain modules
  - Option module
- Undo / redo tree
- Library catalog

# Forward compatibility things
- patchs should have spots for multiple graphs
- patchs should store tags, just a list of strings
- Builtin modules can use the same save syntax as other modules, using one or more complex controls to
  store their extra data.
