# Custom Modules

Modules are defined by two files that look something like this:

```julia
# MODULE_NAME.module.jl
function exec()
    output = input .* gain
end
```
```yaml
# MODULE_NAME.module.yaml
save_id: 16
outputs:
  output:
    datatype: audio
    label: Output
    tooltip: The amplified audio
controls:
  input:
    type: Input
    datatype: audio
  gain:
    type: FloatInRange
    min: 0
    max: 4
    default: 1
gui:
  label: Amplifier
  category: Utility
  tooltip: Changes the volume of an audio signal (optionally decreasing it)
  width: 2
  height: 2
  widgets:
    - type: Input
      y: 0
      control: input
      label: Input
      tooltip: The audio to be amplified
    - type: Knob
      x: 0
      y: 0
      control: gain 
      label: Gain
      tooltip: How much gain to apply
```

The `.module.jl` file is written in a language called [Julia](https://julialang.org/)
which is designed for fast mathematical processing. It has a variety of features
that come in handy when doing digital signal processing. The file itself 
describes the actual algorithm your module should perform.

The `.module.yaml` file describes metadata about the module such as what its
name is and what its outputs are. Note how the output named `output` and the
controls named `input` and `gain` are referenced by name in the `.jl` file.
