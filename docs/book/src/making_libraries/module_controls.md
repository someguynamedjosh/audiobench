# List Of Controls
**FloatInRange** and **Input** are particularly common and good starting points.

## Duration
```yaml
type: Duration
# Optional. Values are "decimal" or "fractional". Default is "decimal".
default_format: fractional 
# Optional. Write a floating point number if using "decimal" format.
default: 3/8
```
Represents an amount of time. Its Julia type is `ControlSignal`. It works best
with the `DurationBox` widget.

## FloatInRange
```yaml
type: FloatInRange
# Required. The lowest value this control can be set to.
min: -3.5
# Required. The highest value this control can be set to.
max: 4.1
# Optional. Default is whatever value you put for min.
default: 0.0
# Optional. Appended after the value of this control in all visual representations.
suffix: kg
```
Probably the most common type of control. Its Julia type is `ControlSignal`. It
can be automated by other audio signals. It is commonly represented by the
`Knob` widget, which the user can drag around to select a value between `min`
and `max`. There are several other widgets that work with this control, all
providing the same functionality just in different form factors.
> NOTE: Due to the fact that these controls can be automated by audio signals
> and that audio signals can have values with magnitude greater than `1.0`, the
> value of the control can end up being outside the range specified by `min` and
> `max`.

## Frequency
```yaml
type: Frequency
# Optional. Default is 1Hz.
default: 440.0
```
Represents a user-selectable frequency. Its Julia type is `ControlSignal`. Works
with the `FrequencyBox` widget.

## Input
```yaml
type: Input
# Required. Values are "audio", "pitch", "trigger", "waveform".
datatype: audio
# Optional. Values vary depending on the selected datatype. Using an invalid
# option will generate an error containing a list of available options.
default: silence
```
An input connection that wires can be connected to. The resulting Julia type is
dependent on the `datatype` selected. For `audio`, it is `StereoAudio`. For
`pitch`, it is `ControlSignal`. For `trigger`, it is `Trigger`. For `waveform`,
it is `Waveform`. The default options available also change based on the
datatype. For `audio`, the only option is `Silence`. For `pitch`, the only
option is `Note Pitch`. For `trigger`, the options are `Note Start`,
`Note Release`, and `Never`. For `Waveform`, the options are `Silence`,
`Ramp Up`, `Ramp Down`, and `Sine Wave`.

## Int
```yaml
type: Int
# Required. The lowest value this control can be set to.
min: -3
# Required. The highest value this control can be set to.
max: 4
# Optional. Default is whatever value you put for min.
default: 2
```
Provides an integer. Its Julia type is `Int32`.

## OptionChoice
```yaml
type: OptionChoice
# Required. A list of options the user can pick from. The names of these options
# are used by the OptionBox widget. You must have at least two options.
options:
- Option 1
- Option 2
- Option 3
# Optional. An index of an option to be selected by default. Default is 0.
default: 2 # Option 3.
```
Provides a selection for the user to pick from. Its Julia type is `Int32`,
indicating the index of the selected item. Note that unlike Julia's arrays, this
index will be zero-based. (Audiobench's core engine is written in Rust which 
uses zero-based indexing.)

## TimingMode
```yaml
type: TimingMode
# Optional. Values are "note" or "song". "note" means the timing is relative to
# the start of the note instead of the start of the song. Default is note.
default_source: song
# Optional. Values are "seconds" and "beats". Default is seconds.
default_units: beats
```
Allows a user to pick how timing should work for a module, whether it should
be relative to the start of a note or the start of a song, and whether time
should be measured in seconds or beats (which will change the timing based
on bpm.) It can be used in code like this:
```julia
# timing is a ControlSignal
timing = get_timing(context, name_of_timing_mode_control)
for s in sample_indices(MonoAudio)
    time_now = timing[%, 1, s]
    lfo_value = sin(time_now)
end
```

## TriggerSequence
```yaml
type: TriggerSequence
```
Allows picking a length and a pattern of boolean values of that length. Its
Julia type is `Vector{Bool}`.

## ValueSequence
```yaml
type: ValueSequence
```
Allows picking a length and a pattern of numeric values of that length. Its
Julia type is `Vector{Float32}`. Each value is between `-1f0` and `1f0`.

