mutable struct StaticData
    start::Float32
    releasing::Bool
    last_value::Float32
end

function static_init()
    return StaticData(-1000f0, false, 0f0)
end

function exec()
    signal = similar(MonoAudio)
    timing = get_timing(context, timing_mode)

    for s in sample_indices(MonoAudio)
        if !static.releasing
            if reset_trigger[1, s]
                static.start = timing[1, s]
            end
            if release_trigger[1, s]
                static.start = timing[1, s]
                static.releasing = true
            end
        end

        now = timing[1, s] - static.start
        value = Float32(0)
        if static.releasing
            if now < release_time[1, s]
                value = lerp(static.last_value, 0f0, now / release_time[1, s])
            else
                value = 0f0
            end
        else
            if now < attack_time[1, s]
                value = now / attack_time[1, s]
            else
                now = now - attack_time[1, s]
                if now < decay_time[1, s]
                    value = lerp(1f0, sustain[1, s], now / decay_time[1, s])
                else
                    value = sustain[1, s]
                end
            end
            static.last_value = value
        end

        signal[1, s] = value * 2f0 - 1f0
    end

    if do_feedback
        now_time = timing[1, 1] - static.start
        if static.releasing
            now_time = now_time + attack_time[1, 1] + decay_time[1, 1]
            if now_time > attack_time[1, 1] + decay_time[1, 1] + release_time[1, 1]
                now_time = attack_time[1, 1] + decay_time[1, 1] + release_time[1, 1]
            end
        elseif now_time > attack_time[1, 1] + decay_time[1, 1]
            now_time = attack_time[1, 1] + decay_time[1, 1]
        end
        multiplier = 1f0
        if timing_mode_unit_is_beats(timing_mode)
            multiplier = 60.0 / context.global_in.bpm
        end
        push!(graph_feedback, first(attack_time) * multiplier)
        push!(graph_feedback, first(decay_time) * multiplier)
        push!(graph_feedback, first(sustain))
        push!(graph_feedback, first(release_time) * multiplier)
        push!(graph_feedback, now_time * multiplier)
        push!(graph_feedback, first(signal))
    end
end
