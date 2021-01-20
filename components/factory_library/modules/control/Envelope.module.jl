mutable struct StaticData
    start::Float32
    releasing::Bool
    last_value::Float32
end

function static_init()
    return StaticData(0f0, false, 0f0)
end

function exec()
    signal = similar(MonoAudio)

    timing = get_timing(context, timing_mode)
    for i in sample_indices(signal)
        if !static.releasing
            if reset_trigger[%, 1, i]
                static.start = timing[1, i]
            end
            if release_trigger[%, 1, i]
                static.start = timing[1, i]
                static.releasing = true
            end
        end

        now = timing[1, i] - static.start
        value = Float32(0)
        if static.releasing
            if now < release_time[%, 1, i]
                value = static.last_value * (1f0 - now / release_time[%, 1, i])
            else
                value = 0f0
            end
        else
            if now < attack_time[%, 1, i]
                value = now / attack_time[%, 1, i]
            else
                now = now - attack_time[%, 1, i]
                if now < decay_time[%, 1, i]
                    value = 1f0 - now / decay_time[%, 1, i] * (1f0 - sustain[%, 1, i])
                else
                    value = sustain[%, 1, i]
                end
            end
            static.last_value = value
        end

        signal[1, i] = value * 2f0 - 1f0
    end

    if do_feedback
        now_time = timing[1, 1] - static.start;
        if static.releasing
            now_time = now_time + attack_time[%, 1, 1] + decay_time[%, 1, 1];
            if now_time > attack_time[%, 1, 1] + decay_time[%, 1, 1] + release_time[%, 1, 1]
                now_time = attack_time[%, 1, 1] + decay_time[%, 1, 1] + release_time[%, 1, 1];
            end
        elseif now_time > attack_time[%, 1, 1] + decay_time[%, 1, 1]
            now_time = attack_time[%, 1, 1] + decay_time[%, 1, 1];
        end
        multiplier = 1f0
        if timing_mode_unit_is_beats(timing_mode)
            multiplier = 60.0 / global_bpm;
        end
        push!(graph_feedback, first(attack_time) * multiplier)
        push!(graph_feedback, first(decay_time) * multiplier)
        push!(graph_feedback, first(sustain))
        push!(graph_feedback, first(release_time) * multiplier)
        push!(graph_feedback, now_time * multiplier)
        push!(graph_feedback, first(signal))
    end
end
