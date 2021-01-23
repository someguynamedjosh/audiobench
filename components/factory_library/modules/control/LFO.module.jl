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