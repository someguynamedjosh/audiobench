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
            if reset_trigger[i]
                static.start = timing[i]
            end
            if release_trigger[i]
                static.start = timing[i]
                static.releasing = true
            end
        end

        now = timing[i] - static.start
        value = Float32(0)
        if static.releasing
            if now < release_time
                value = static.last_value * (1f0 - now / release_time)
            else
                value = 0f0
            end
        else
            if now < attack_time
                value = now / attack_time
            else
                now = now - attack_time
                if now < decay_time
                    value = 1f0 - now / decay_time * (1f0 - sustain[1, i])
                else
                    value = sustain[1, i]
                end
            end
            static.last_value = value
        end

        signal[i] = value * 2f0 - 1f0
    end

    if context.global_in.do_update
        now_time = first(timing) - static.start;
        if releasing
            now_time = now_time + attack_time + decay_time;
            if now_time > attack_time + decay_time + release_time
                now_time = attack_time + decay_time + release_time;
            end
        elseif now_time > attack_time + decay_time
            now_time = attack_time + decay_time;
        end
        multiplier = 1f0
        if TimingModeIsBeatSynchronized(TIMING_MODE)
            multiplier = 60.0 / global_bpm;
        end
        SetGraphFeedback([
            attack_time * multiplier,
            decay_time * multiplier,
            first(sustain),
            release_time * multiplier,
            now_time * multiplier,
            first(signal),
        ]);
    end
end
