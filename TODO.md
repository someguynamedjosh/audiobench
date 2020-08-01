# Things with no dependencies
- Straighten out color names
- Make Nodespeak errors use squiggles instead of colors
- Better way to pass extra data to module widget draw() functions
- Make website look nice
- Make window resizable

# Things with missing dependencies
- Variable range for pitch wheel
- Optional variable smoothing for MIDI controls

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
- Higher precision for MIDI controls
- Optional smoothing for MIDI controls
- Things for making waveforms using harmonics of other waveforms
- Update tooltip when clicking on timing control
- Line artifact when rendering default inputs
- Better integration with other VST library preset methods
- Dependency checks between libraries
- Hint that enter can be pressed in typing widget
- Different cursors on hover
- Crossfade module
- Say specifically which library unwritable patches come from, not just \[factory\].
- Comb filter
- Fuzz testing to ensure save_data deserialization always exits gracefully on corrupt data
- Add type bounds for AUTO variables

# Code organization stuff
- Use more TupleUtil functions
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
- Library browser

# Forward compatibility things
