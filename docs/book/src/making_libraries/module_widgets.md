# List Of Widgets
There are some properties that many widgets have that serve the same function.
First, almost every widget has an `x` and `y` property, which specifies the
position of the top-left corner of the widget inside the module in grid units.
Note that this is not pixels! For example, a knob is 2 grid units wide which,
when rendered on the screen, takes up about 60 pixels. Many widgets also have
`label` and `tooltip` properties. The former is a piece of text which should be
displayed over or under the widget describing its function. The latter is a
piece of text to display in the top bar when the widget is hovered over. Any
widget which serves as a visual representation of a control will have a
`control` property where you must type the name of the control that the widget
should represent. An error will be generated if the control is not of the
correct type. For example, a `DurationBox` widget cannot be used to represent a
`FloatInRange` control.

## DurationBox
```yaml
type: DurationBox
x: 0
y: 0
duration_control: control_name # Must be a Duration control
mode_control: control_name # Must be a TimingMode control
label: The Duration
tooltip: Controls the duration
```
Represents a `Duration` control. The provided `TimingMode` control is used to
pick whether the suffix 's' or 'b' is displayed to indicate the duration is 
measured in seconds or beats.

## EnvelopeGraph
```yaml
type: EnvelopeGraph
x: 0
y: 0
w: 5
h: 2
feedback_name: graph_feedback_name
```
Displays the shape of an envelope to the user. Feedback can be sent from Julia
code like this:
```julia
push!(graph_feedback_name, attack_time)
push!(graph_feedback_name, decay_time)
push!(graph_feedback_name, sustain)
push!(graph_feedback_name, release_time)
push!(graph_feedback_name, now_time)
push!(graph_feedback_name, now_value)
```

## FrequencyBox
```yaml
type: FrequencyBox
x: 0
y: 0
control: control_name # Must be a Frequency control
label: The Frequency
tooltip: Controls the frequency
```
Represents a `Frequency` control, allowing the user to modify it 
logarithmically, I.E. the same mouse movement is required to go from 1.0Hz to
1.1Hz as going from 1.0kHz to 1.1kHz.

## HSlider
```yaml
type: HSlider
x: 0
y: 0
w: 5 # Width
h: 1 # This should always be 1
control: control_name # Must be a FloatInRange control
label: The Value
tooltip: Controls the value
```
A horizontal slider, allowing a user to edit the value of or connect automation
to a `FloatInRange` control.

## Input
```yaml
type: Input
# You do not need to specify X.
y: 0
control: control_name # Must be an Input control
label: The Input
tooltip: Data that comes into the module for processing
# Optional. The name of an icon to display to identify this input. If this is
# not provided, a default icon is selected based on the datatype of the input
# control, E.G. Factory:waveform is used for waveform inputs.
icon: Factory:save
```
Represents an input by displaying a box on the left-hand side of the module.
Note that you do not need to specify an x coordinate, this is selected
automatically. This widget allows connecting wires to inputs as well as
selecting the default value of the input if no wire is connected.

## IntBox
```yaml
type: IntBox
x: 0
y: 0
control: control_name # Must be an Int control
label: The Integer
tooltip: Controls the integer
```
Visual representation of an `Int` control that allows clicking or dragging to
change its value.

## Knob
```yaml
type: Knob
x: 0
y: 0
control: control_name # Must be a FloatInRange control
label: The Control
tooltip: Controls the control
```
Represents a `FloatInRange` control, allowing changing the un-automated value
as well as connecting and editing automation.

## MiniKnob
```yaml
type: MiniKnob
x: 0
y: 0
control: control_name # Must be a FloatInRange control
label: The Control
tooltip: Controls the control
```
A smaller version of `Knob`, 1x1 instead of 2x2.

## OptionBox
```yaml
type: OptionBox
x: 0
y: 0
w: 3 # Width
h: 4 # Height
control: control_name # Must be an OptionChoice control
label: The Option
tooltip: Controls the option
```
Represents an `OptionChoice` control by displaying all option names in a
vertical list, changing the selected option whenever an entry is clicked.

## OptionIconGrid
```yaml
type: OptionIconGrid
x: 0
y: 0
w: 3 # Width
h: 4 # Height
control: control_name # Must be an OptionChoice control
# Required. A list of icons to display instead of the names of each option.
icons:
- Factory:sine_wave
- Factory:square_wave
label: The Option
tooltip: Controls the option
```
Fulfills the same function as `OptionBox` but displays options as icons instead
of a grid instead of text entries in a vertical list. The `icons` property must
contain an icon for every option the `OptionChoice` control contains.

## TimingSelector
```yaml
type: TimingSelector
x: 0
y: 0
control: control_name # Must be a TimingMode control
```
Represents a `TimingMode` control, allowing the user to toggle between
note-relative and song-relative timing as well as between measuring time in
beats or seconds.

## TriggerSequence
```yaml
type: TriggerSequence
x: 0
y: 0
w: 4 # Width
h: 1 # Height, should always be 1.
control: control_name # Must be a TriggerSequence control
tooltip: Controls the trigger sequence
# Required.
feedback_name: playhead_feedback
```
Allows a user to input a boolean sequence into a `TriggerSequence` control. The
feedback should contain a single floating point value indicating a position to
place a playhead marker. The units are steps, so providing a value of `1.5f0` 
will place the marker above the middle of the second step of the sequence. In
code, that would look like this:
```julia
push!(playhead_feedback, 1.5f0);
```

## TriggerSequenceLength
```yaml
type: TriggerSequenceLength
x: 0
y: 0
control: control_name # Must be a TriggerSequence control
label: The Length
tooltip: Controls the length
```
Controls the length of a `TriggerSequence`'s sequence.

## ValueSequence
```yaml
type: ValueSequence
x: 0
y: 0
w: 4 # Width
h: 1 # Height, should always be 1.
sequence_control: control_name # Must be a ValueSequence control
tooltip: Controls the trigger sequence
# Required.
feedback_name: playhead_feedback
```
Allows a user to input a boolean sequence into a `ValueSequence` control. The
feedback should contain two floating point values, the indicating a position to
place a playhead marker. The units are steps, so providing a value of `1.5f0` 
will place the marker above the middle of the second step of the sequence. The
second value is how much ramping to display. In code, that would look like this:
```julia
push!(playhead_feedback, 1.5f0);
push!(playhead_feedback, ramping); # ramping is between 0f0 and 1f0
```

## ValueSequenceLength
```yaml
type: ValueSequenceLength
x: 0
y: 0
sequence_control: control_name # Must be a ValueSequence control
label: The Length
tooltip: Controls the length
```
Controls the length of a `ValueSequence`'s sequence.

## WaveformGraph
```yaml
type: WaveformGraph
x: 0
y: 0
w: 4 # Width
h: 4 # Height
feedback_name: graph_feedback
```
Renders a list of floating point values as a waveform. Additionally renders a 
crosshair using the first two values. In code, it would look like this:
```julia
push!(graph_feedback, phase_now)
push!(graph_feedback, waveform(phase_now, 1))
for s in 1:40
    push!(graph_feedback, waveform(s / 40f0, 1))
end
```
