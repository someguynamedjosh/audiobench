mutable struct StaticData
    base_time::Float32
end

function static_init()
    StaticData(0.0)
end

function exec()
    out_value = similar(MonoAudio)
    num_steps = Int32(length(sequence))
    timing = get_timing(context, timing_mode)

    if do_feedback
        value = (first(timing) - static.base_time) / first(step_time) % Float32(num_steps)
        push!(playhead_feedback, value)
        push!(playhead_feedback, first(ramping))
    end

    for s in sample_indices(Trigger)
        if reset[1, s]
            static.base_time = timing[1, s]
        end
        sequence_time = (timing[1, s] - static.base_time) / first(step_time) % Float32(num_steps)
        step_index = floor(Int32, sequence_time)
        step_progress = sequence_time % 1f0
        ramp_start = 1f0 - ramping[1, s]
        if step_progress <= ramp_start
            # Every time I have to add a +1 I die a little inside.
            out_value[1, s] = sequence[step_index + 1]
        else
            next_index = (step_index + Int32(1)) % num_steps
            ramp_amount = (step_progress - ramp_start) / ramping[1, s]
            out_value[1, s] = lerp(sequence[step_index + 1], sequence[next_index + 1], ramp_amount)
        end
    end
end
