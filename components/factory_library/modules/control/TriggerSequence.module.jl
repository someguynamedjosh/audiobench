mutable struct StaticData
    base_time::Float32
    last_step::Int32
end

function static_init()
    StaticData(0.0, -1)
end

function exec()
    out_trigger = similar(Trigger)
    num_steps = length(sequence)
    timing = get_timing(context, timing_mode)

    if do_feedback
        value = (first(timing) - static.base_time) / first(step_time) % Float32(num_steps)
        push!(playhead_feedback, value)
    end

    for s in sample_indices(Trigger)
        if reset[%, 1, s]
            static.base_time = timing[%, 1, s]
        end
        current_step = 
            floor(Int32, (timing[%, 1, s] - static.base_time) / first(step_time)) % num_steps
        if static.last_step != current_step
            static.last_step = current_step
            out_trigger[1, s] = sequence[current_step + 1] # grumble grumble
        else
            out_trigger[1, s] = false
        end
    end
end
