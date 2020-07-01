# Things with no dependencies
- Add ctrl / shift hints to the mouse hint indicator thing
- Allow editing the ends of a maxed-out automation lane by clicking outside the 
  bounds of the lane as if it were a full circle instead of a half circle.
- Rename 'base' to 'factory'
- Add cursor to waveform display (Use in LFO module)
- Fix new patch (created with + button) not showing up on list
- Make inputs not change default when you try to connect them to something but they don't connect to anything
- Why does it randomly stop letting me connect things?!?!?!?!
- Support saving/loading through VST API so that it works in DAWs and such

# Things with missing dependencies
- Trigger sequencer
- Pitch sequencer
- Variable range for pitch wheel

# Low-priority things without dependencies
- Int boxes are a pain to use because double-click
- Output silent audio while recompiling instead of hanging the thread
- Get a better icon for waveforms
- Better nothing icon
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
- Some kind of library identification file with information like the minimum version of audiobench you
  need to use the library
