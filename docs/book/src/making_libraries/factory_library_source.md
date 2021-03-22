# Factory Library Source Code

For people like me who learn best with examples, the source code for the entire
factory library can be viewed
[here on GitHub](https://github.com/joshua-maros/audiobench/tree/latest-beta/components/factory_library).
Here are also a couple examples taken out of that library:

## The Diesel Bass Patch
```
AgsARGllc2VsIEJhc3MBBwBGYWN0b3J5IAINAAIQ____QP___wAAyAEAADAAAAAAAWAAAAAYAAAAAA-A_v__MAAAAAAE0Pz__5AAAAAAA6j9__8wAAAAAA3Q____MAAAAAANEP___8AAAAAAAvD9__-oAAAAAAFw____6P___wALgP7__7j___8ACQgBAAAwAAAAAAFgAAAAYAAAAEAQrgaAkvXoDoF_WzzoLof4XAAAAAg0ABMAAAD8_y8A4f8_AQAA-P9XuGhm2v9_AgAA8P9PuI7rqXAdAIAeRQAAcj0CACgKBwAgZiYjMx8AoAACtt2ABCBwFHIAzAMwE0C4jh61MRGAHlWPKiCOwgCgcD26QyrQOg_rhTM-FQAQMzPA_7__fwAAAAAKAPD_nwEAAEBgAgAAgP9_
```

## The Harmonic Module
```yaml
# modules/waveform/Harmonic.module.yaml

save_id: 14
outputs:
  out_wave:
    datatype: waveform
    label: Output
    tooltip: A harmonic of the input waveform
controls:
  base_wave:
    type: Input
    datatype: waveform
    default: sine_wave
  harmonic: 
    type: Int
    min: 1
    max: 99
gui:
  label: Harmonic
  category: Waveform
  tooltip: Creates a waveform which is a harmonic of the input waveform
  width: 2
  height: 2
  widgets:
    - type: Input
      y: 0
      control: base_wave
      label: Input
      tooltip: The base waveform
    - type: IntBox
      x: 0
      y: 0
      control: harmonic
      label: Harmonic
      tooltip: How many times to repeat the input waveform
```
```julia
# modules/waveform/Harmonic.module.jl

function exec()
    out_wave = function (phase::Float32, buffer_pos::Integer)
        base_wave((phase * harmonic) % 1f0, buffer_pos)
    end
end
```

## The LFO Module
```yaml
# modules/control/LFO.module.yaml

save_id: 4
outputs:
  audio:
    datatype: audio
    label: Signal 
    tooltip: Signal output
controls:
  waveform:
    type: Input
    datatype: waveform
    default: sine_wave
  strength:
    type: FloatInRange
    min: 0
    max: 1
    default: 1
  offset:
    type: FloatInRange
    min: -1
    max: 1
    default: 0
  timing_mode:
    type: TimingMode
  cycle_time:
    type: Duration
    default: 1.0
  strength_mode:
    type: OptionChoice
    options:
      - Max
      - Mid
      - Min
    default: 2
gui: 
  label: LFO
  category: Control
  tooltip: Low Frequency Oscillator, used for controlling values that should cycle over time
  width: 8
  height: 4
  widgets:
    - type: Input
      y: 0
      control: waveform
      label: Waveform
      tooltip: The waveform to repeatedly play
    - type: TimingSelector
      x: 4
      y: 2
      control: timing_mode
    - type: DurationBox
      x: 6
      y: 2
      duration_control: cycle_time
      mode_control: timing_mode
      label: Cycle Time
      tooltip: How long the LFO takes to cycle through the waveform once
    - type: WaveformGraph
      x: 4
      y: 0
      w: 4
      h: 2
      feedback_name: graph_feedback
    - type: Knob
      x: 2
      y: 2
      control: offset
      label: Offset
      tooltip: Where in the waveform the LFO should start playing
    - type: Knob
      x: 2
      y: 0
      control: strength
      label: Strength
      tooltip: How strong the output should be
    - type: OptionBox
      x: 0
      y: 0
      w: 2
      h: 3
      control: strength_mode
      label: Mode
      tooltip: How the strength knob should affect the waveform
```
```julia
# modules/control/LFO.module.jl

function apply_strength(value::Float32, strength::Float32, mode::Integer)
    if mode == 0
        value * strength + (1f0 - strength)
    elseif mode == 1
        value * strength
    else
        @assert mode == 2
        value * strength - (1f0 - strength)
    end
end

function exec()
    audio = similar(MonoAudio)
    timing = get_timing(context, timing_mode)

    for s in sample_indices(MonoAudio)
        phase = (timing[%, 1, s] / cycle_time[%, 1, s] + offset[%, 1, s] + 1f0) % 1f0
        sample = apply_strength(waveform(phase, s), strength[%, 1, s], strength_mode)
        audio[1, s] = sample
    end

    if do_feedback
        offset = last(offset)
        phase = last(timing) / last(cycle_time)
        push!(graph_feedback, (phase + 2f0) % 1f0)
        push!(graph_feedback, last(audio))
        for s in 1:default_graph_resolution
            phase = ((s - 1) / Float32(default_graph_resolution - 1) + offset) % 1f0
            sample = apply_strength(waveform(phase, s), strength[%, 1, s], strength_mode)
            push!(graph_feedback, sample)
        end
    end
end
```