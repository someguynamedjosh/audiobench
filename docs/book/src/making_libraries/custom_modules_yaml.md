# The .module.yaml File

The `.module.yaml` file describes metadata about the module. Among other things,
it allows Audiobench to create a graphical representation of the model and route
the appropriate data in to and out of the `exec()` function defined in the
corresponding `.module.jl` file. The file should follow this format:

```yaml
save_id: 0
outputs:
    [List of outputs]
controls:
    [List of controls]
gui:
  label: Thing Doer
  category: Utility
  tooltip: Does things
  width: 10
  height: 10
  widgets:
    [List of widgets]
```

## `save_id`
The `save_id` must be unique among all the modules in a particular library. If
it is not, opening Audiobench will produce an error recommending the next
available unused ID. Because of this it is recommended to set the ID to 0 when
making a new module, then change it to the value that Audiobench recommends
on startup.

## `outputs`
The outputs list is formatted like this:
```yaml
outputs:
    output_name:
        datatype: [audio, pitch, trigger, or waveform]
        label: Output
        tooltip: Where the output comes out of.
```
The datatype specifies what kind of data the output will carry. Note that the
actual Julia datatype of pitch data is just `ControlSignal` (or possibly
`StaticControlSignal`.) `label` specifies a piece of text to show next to the
output when it is being hovered over. `tooltip` specifies tooltip text to
display in the top bar when the output is being hovered over. To use this
output in your Julia code, you operate on a variable named `output_name`. For
example:
```julia
output_name = Similar(StereoAudio)
```
Note that you *must* create a value and assign it to this variable. If your
`exec()` function completes without defining a variable named `output_name`, you
will get a compiler error.

## `controls`
These are the things the user can modify to control how your module works. Note
that this section does not contain the visual representation of the controls,
that part is done in the `widgets` section. The `controls` section is formatted
like this:
```yaml
controls:
    control_name:
        type: ControlType
        other_data: specific to this ControlType
```
You can access the values of these controls inside the corresponding Julia code
with the variable name you provide in place of `control_name`. A list of the
different control types and the parameters they each require is available in the
next section.

## `gui`
This section tells Audiobench how to construct a visual representation of the
module. The `label` is the name of the module that is displayed in the module
browser and above the module in the module graph when hovering over it. The
`category` is a name under which the module should be placed in the module
browser. There is no limit on what this can be, so be aware that if you misspell
the name of an existing category then your module will show up as the only
module in the new misspelled category. The `tooltip` is text which shows in the
top bar when the module is being hovered over. The `width` and `height` specify
the size of the light gray area of the module where widgets can be placed. You
do not need to take into account the extra space needed for the dark blue bars
on either side of the module.

## `widgets`
This section specifies what widgets should be shown on the module. These are
usually used either to show a visual representation of a control or to display
some feedback data sent from the `exec()` function to the user in a readable
format. The section is formatted like this:
```yaml
gui:
  widgets:
  - type: WidgetType
    x: 0
    y: 0
```
A list of all current widgets is provided in a following chapter. There are some
properties that many widgets have that serve the same function. First, almost
every widget has an `x` and `y` property, which specifies the position of the
top-left corner of the widget inside the module in grid units. Note that this is
not pixels! For example, a knob is 2 grid units wide which, when rendered on the
screen, takes up about 60 pixels. Many widgets also have `label` and `tooltip`
properties. The former is a piece of text which should be displayed over or
under the widget describing its function. The latter is a piece of text to 
display in the top bar when the widget is hovered over. Any widget which serves
as a visual representation of a control will have a `control` property where you
must type the name of the control that the widget should represent. An error 
will be generated if the control is not of the correct type. For example, a
`DurationBox` widget cannot be used to represent a `FloatInRange` control.
