# Things with no dependencies
- Fix rendering of controls that have a zero point outside of their range
- More default waveforms (ramp up/down, sin)
- More snapping points when dragging knobs
- Global timing and tempo
- Get rid of JUCE logo
- Show error to user on panic?
- Don't unwrap the engine in the main instance, only map.
- Interpolation for MIDI parameters
- Add ctrl / shift hints to the mouse hint indicator thing

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
