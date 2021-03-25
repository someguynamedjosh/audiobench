# The .module.jl File

The `.module.jl` file is written in a language called [Julia](https://julialang.org/)
which is designed for fast mathematical processing. It has a variety of features
that come in handy when doing digital signal processing. The file itself 
describes the actual algorithm your module should perform.

## File contents

Audiobench will look for three items defined in this file:
```julia
# Optional
mutable struct StaticData
    echo_buffer::StereoAudio
end

# Optional
function static_init()
    echo_buffer = similar(StereoAudio)
    StaticData(echo_buffer)
end

# Required
function exec()
    output .= input .+ static.echo_buffer
    static.echo_buffer .= input
end
```

You can also define any additional items you want to help write your module such
as constants or helper functions.

The `exec()` function is required and is called many times per second whenever
your module needs to process audio. Audiobench will automatically pass several
variables to this function based on the contents of your `.module.yaml` file.
The necessary arguments are automatically inserted for you so you do not have
to worry about manually writing them in the correct order.

> WARNING: At the moment, manually returning from the exec() function will cause
> an error, so make sure your control flow always reaches the end of the
> function!

The `StaticData` struct is optional and defines a data structure that can hold
on to data between multiple calls of `exec()`. For example, the Oscillator
module has a StaticData struct that remembers the last phase that was outputted
so that when `exec()` is called again, it can pick up where it left off.
