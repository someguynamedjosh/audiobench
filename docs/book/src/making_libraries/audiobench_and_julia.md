# Audiobench-Specific Julia Patterns

This section will go over some coding patterns that are helpful specifically
when developing Audiobench modules as well as helpful functions and types added
by the factory library. If you would like to see the actual Julia file defining
most of what's discussed in this page, you can check it out
[on the GitHub repository](https://github.com/joshua-maros/audiobench/blob/latest-beta/components/factory_library/lib.lib.jl)

## Types

In Audiobench, the main data types are *audio signals*, *control signals*, 
*triggers*, and *waveforms*. Creating a module will involve consuming or producing
at least one of these types of data. The way these are defined and used in Julia
is as follows:

### Audio
```julia
# StereoAudio contains audio data that changes over time and has different
# content between its two channels. It is a type alias for a static array of
# Float32s.
audio = similar(StereoAudio)
audio[channel, sample] = 0f0
# MonoAudio is like StereoAudio but only has one channel
mono_audio = similar(MonoAudio)
audio[1, sample] = 0f0 # Good.
audio[2, sample] = 0f0 # RUNTIME ERROR!
# There are also 'static' variants that hold audio data which does not change.
StaticMonoAudio
StaticStereoAudio
```

### Control Signals
```julia
# Control signals are values that change over time but do not have multiple 
# channels. They are just aliased to Audio types for simplicity. This is the
# datatype that a knob widget would control. Here the presence of a static
# version makes more sense, as controls can be set by the user to an unchanging
# value and left alone, so the static version has a lot of potential for 
# optimization since it is only carrying one copy of that value instead of 512
# copies.
@assert ControlSignal == MonoAudio
@assert StaticControlSignal == StaticMonoAudio
```

### Triggers
```julia
# Triggers are boolean arrays. Any place where the array is true is a place
# where a trigger was sent. Like other data types, there is a static version.
trigger = similar(Trigger)
# Only 1 channel
trigger[1, sample] = true
static = similar(StaticTrigger)
# This is usually the only use for a static trigger. One which was true would
# be considered to be firing on every single sample.
static[1, 1] = false
```

### Waveforms
```julia
# Waveforms are actually just functions, which are not strongly typed by their
# argument and return types in Julia. Currently there is no robust check in
# place to make sure that places where Audiobench expects a valid waveform are
# actually being given a valid waveform.
waveform = function(phase::Float32, buffer_pos::Integer)
    sin(phase + some_control[%, 1, buffer_pos])
end
# This is not a valid waveform. It's going through a rebellious phase.
not_waveform = function(something::String, irrelevant::Float64)
    [1, 2, 3, 4, 5]
end
```

## Constants

These constants are available anywhere without having to explicitly import
anything.
```julia
channels::Integer      # How many channels the output has. At the moment 2 is the
                       # supported value.
buffer_length::Integer # How many samples are held in a buffer and consequently,
                       # how many samples are processed by a call to exec().
sample_rate::Float32   # How many samples occur per second.
```
For example, you can get the amount of time that should pass in the audio 
during a call of exec() by doing:
```julia
buffer_lenght #= samples per buffer =# / sample_rate #= samples per second =#
```
If you want to iterate over a buffer, you should use `sample_indices(buffer)`
and `channel_indices(buffer)` rather than manually looping over the range
`1:buffer_length` and `1:channels`. Read the section **Consuming Data** for more
information.

## Producing Data
One way to produce audio, control, or trigger data is to use `similar()` to
create a mutable buffer that is later written to:
```julia
output = similar(MonoAudio)
for s in sample_indices(MonoAudio)
    output[1, s] = Float32(s)
end
```
If your data is dependent on a number of controls on your module, it is a good
idea to let Julia figure out the data type it needs:
```julia
# By programming it this way instead of by manually creating a buffer, it opens
# it up to heavy optimization. The best case is if both `control1` and 
# `control2` are of a type like `StaticMonoAudio`, then `intermediate_value`
# will also be of type `StaticMonoAudio`. As soon as even one of the controls
# becomes something bigger like `MonoAudio`, the output type is also correctly
# changed to `MonoAudio`. Julia's vectorization system is even smart enough to
# handle the case where one control is of type `StaticStereoAudio` and the other
# is of type `MonoAudio`, correctly producing a result of type `StaticAudio`.
# Note that we do NOT use .= here because intermediate_value does not yet
# contain an array that can be written to.
intermediate_value = control1 .+ sin.(control2)
```
Both audio and control signals should contain floating point values in the range
of `-1f0` to `1f0`. Note that this limit is not currently enforced, for example
a user could take a signal that reaches these limits and then route it through
an amplifier, making it peak at `2f0`. Because of this, if your algorithm 
absolutely requires values to be in range, make sure to clip them before
using them.

Triggers are arrays of boolean values. Anywhere that the array contains `true`
is a place where a trigger should be considered to have fired.

Waveforms are the odd data type out. Unlike the other three data types, they
are functions instead of arrays. They accept two parameters, the first is a
phase and the second is a buffer position. An example definition of a sine wave:
```julia
# Julia handles the unicode symbol π just fine which I think is super neat.
waveform = function(phase::Float32, buffer_pos::Integer)
    sin(phase * 2π)
end
```
The `buffer_pos` argument is there if you want to include some controls which
can be automated by the user in the definition of your waveform:
```julia
square_wave = function(phase::Float32, buffer_pos::Integer)
    if phase < duty_cycle[%, 1, buffer_pos] 1f0 else -1f0 end
end
```
Unlike with control signals and audio, the `phase` argument *must* be between
`0f0` and `1f0`, so you can safely write equations that depend on this to be
true.

## Consuming Data
For data types like audio, control signals, and triggers, your module code may
be given any of the possible data types for that kind of information. It is then
important to account for this possibility. For example, consider this code:
```julia
second_sample = some_control[1, 2]
```
This could fail if `some_control` is one of the `Static` variants which only
contains a single value, indicating that it did not change during the span of 
time it represents. The factory library extends indexing in a custom way to make
situations like this easier. By adding a percent sign `%` as the first index,
any Audiobench-related data can be accessed as if it were the biggest possible
type for that kind of data. For example:
```julia
full_audio_data = similar(StereoAudio)
# Note that the % trick won't work on the left hand side of an equal sign, as
# its behavior would be misleading.
full_audio_data[2, 3] = 1f0
@assert full_audio_data[%, 2, 3] == 1f0
# This represents audio data which is the same on all channels and during the
# entire 'length' of the buffer, only storing a single value.
static_audio_data = similar(StaticMonoAudio)
static_audio_data[1, 1] = 1f0
# Access it as if it were stereo and changed over time. Any access will return
# the same value.
@assert static_audio_data[%, 2, 3] == 1f0
@assert static_audio_data[%, 2, 19] == 1f0
@assert static_audio_data[%, 1, 1] == 1f0
```
If you want to manually iterate over data, the `channel_indices` and 
`sample_indices` functions are quite useful:
```julia
output = similar(input)
# In memory, stereo audio data is stored so that the channel index changes the
# fastest. In other words, in memory it would look like LRLRLRLRLRLRLRLR and not
# LLLLLLLLLRRRRRRRRR. So it is more cache-friendly (and thus faster) to put the
# channel loop on the inside.
for s in sample_indices(input)
    # If input is mono audio then the loop will only execute once with `c == 1`.
    # If it is stereo, it will repeat twice with `1` and `2`.
    for c in channel_indices(input)
        # Using the percent trick is optional here since we already know for
        # sure that `s` and `c` are valid for this array.
        output[c, s] = 2f0 * input[%, c, s]
    end
end
```
It is often the case where you need to manually iterate over samples but can
still leverage Julia's slicing feature to avoid manually iterating over
channels:
```julia
output = similar(input)
for s in sample_indices(input)
    # The ':' indicates you want a slice from the beginning to the end. In this
    # case, we want a slice of all the channels for the particular sample `s`.
    # Also note the use of the .= since a slice is not a scalar, we want to
    # assign into it instead of over it.
    output[:, s] .= input[%, :, s] .* sin(static.phase)
    # 1.0 / sample_rate is the amount of time represented by a single audio
    # sample.
    static.phase += 1.0 / sample_rate
end
```
Triggers and control signals are accessed in the same way, although trying to
access any channel other than `1` will cause an error as neither of these types
support having multiple channels.
```julia
data = similar(Trigger)
data[1, 2] = true
@assert data[%, 1, 2] == true
static_data = similar(StaticTrigger)
static_data[1, 1] = true
@assert static_data[%, 1, 81] == true
```
Waveforms are functions, so using them only involves passing the correct
arguments. The first argument is the phase you want to look up in the waveform,
in a range between `0f0` and `1f0`. The second parameter is an integer
indicating which sample index this lookup is being performed for. This is
necessary to allow waveforms that change over time. For example, a user could
automate the parameter knob on the Starter Shapes module to produce a waveform
which is constantly changing. This second parameter then informs the waveform of
what time during that automation it should look up its parameters from. The
waveform will return a value of type `Float32`. To demonstrate, here is code for
a simple oscillator:
```julia
output = similar(MonoAudio)
for s in sample_indices(MonoAudio)
    output[1, s] = waveform(static.phase % 1f0, s)
    static.phase += pitch[%, 1, s] / sample_rate
end
```
The value of the phase argument *must* be between `0f0` and `1f0` or you may
get nonsensical results.

## Feedback Data
Some widgets have a property called `feedback_name` which indicates that it can
receive values back from Julia to display in a friendly way to the user. For
example, the `TriggerSequence` widget accepts a floating point value to display
as a playhead along the sequence so the user can see what part of the sequence
is currrently being used by the module. At the moment all feedback data comes
in the form of `Vector{Float32}`s, so code to send feedback will look like this:
```julia
push!(chosen_feedback_name, playhead_progress)
```
It would be inefficient to compute this data during every call to `exec()` since
it is called much faster than the display's framerate, and additionally only one
note's feedback data could ever be viewed at a time. For this reason, there
is a variable available called `do_feedback` which is only occasionally true.
Code to provide feedback data should follow this pattern:
```julia
if do_feedback
    value_to_display = some_complicated_computation()
    push!(chosen_feedback_name, value_to_display)
end
```
This behavior is deliberately a part of the `exec()` function rather than being
contained in a seperate function so that you can display actual data that is
being computed without having to manually store it for when a seperate feedback
function was called.

## Helper Methods
```julia
# Audio type to sample type
@assert at2st(MonoAudio) == MonoSample
# Sample type to audio type
@assert st2at(StereoSample) == StereoAudio
# Sample type to static audio type
@assert st2sat(StereoSample) == StaticStereoAudio
# Audio to control signal, flattens a (potentially stereo) audio signal to a
# (guaranteed mono) control signal.
@assert typeof(a2cs(stereo_audio)) == ControlSignal
@assert ControlSignal == MonoAudio
# Linear interpolation.
@assert lerp(from, to, 1f0) == to
@assert lerp(0f0, 5f0, 0.2f0) == 1f0
# mutable() returns the mutable version of a data type.
mutable struct StaticData
    echo_memory: mutable(StereoAudio)
end
# viewas() lets you treat bigger data like it was smaller data.
echo_memory = viewas(static.echo_memory, typeof(mono_audio))
echo_memory .+= mono_audio
echo_memory .*= 0.5f0
# typeof2() is useful for explicitly getting the Audiobench-defined datatype of
# something since Julia considers the immutable and mutable data types to be 
# different.
@assert typeof(incoming_audio) == mutable(StereoAudio)
@assert typeof(incoming_audio) != StereoAudio
@assert typeof2(incoming_audio) == StereoAudio
@assert at2st(typeof2(incoming_audio)) == StereoSample
```
