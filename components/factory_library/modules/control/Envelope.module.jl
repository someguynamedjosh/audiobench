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

    # if context.global_in.do_update
    #     now_time = first(timing) - static.start;
    #     if releasing
    #         now_time = now_time + attack_time[%, 1, i] + decay_time[%, 1, i];
    #         if now_time > attack_time[%, 1, i] + decay_time[%, 1, i] + release_time[%, 1, i]
    #             now_time = attack_time[%, 1, i] + decay_time[%, 1, i] + release_time[%, 1, i];
    #         end
    #     elseif now_time > attack_time[%, 1, i] + decay_time[%, 1, i]
    #         now_time = attack_time[%, 1, i] + decay_time[%, 1, i];
    #     end
    #     multiplier = 1f0
    #     if TimingModeIsBeatSynchronized(TIMING_MODE)
    #         multiplier = 60.0 / global_bpm;
    #     end
    #     SetGraphFeedback([
    #         attack_time[%, 1, i] * multiplier,
    #         decay_time[%, 1, i] * multiplier,
    #         first(sustain),
    #         release_time[%, 1, i] * multiplier,
    #         now_time * multiplier,
    #         first(signal),
    #     ])
    # end
end
