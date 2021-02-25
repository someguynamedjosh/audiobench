mutable struct StaticData
    old_value::Float32
    new_value::Float32
    old_value_time::Float32
end

function static_init()
    StaticData(0f0, 0f0, 0f0)
end

function exec()
    audio = similar(MonoAudio)
    value_now = 0f0
    factor = 0f0
    timing = get_timing(context, 0) # 0 = note time in seconds.

    for s in sample_indices(MonoAudio)
        delay_now = max_delay[%, 1, s] * delay_mul[%, 1, s]
        time_now = timing[%, 1, s]
        if delay_now <= 1f0 / sample_rate
            static.old_value = static.new_value
            static.new_value = rand()
            value_now = static.old_value
            static.old_value_time = time_now
        else
            if static.old_value_time + delay_now <= time_now
                static.old_value = static.new_value
                static.new_value = rand()
                static.old_value_time += delay_now
            end
            factor = (time_now - static.old_value_time) / delay_now
            if smooth_mode == 0
                value_now = static.old_value
            else
                value_now = static.new_value * factor + static.old_value * (1f0 - factor)
            end
        end
        audio[1, s] = (value_now * 2f0 - 1f0) * amplitude[%, 1, s]
    end

    if do_feedback
        # We use this instead of rand() so that the waveform display doesn't violently flicker
        # every time it is updated.
        dummy_waveform = SA_F32[
            0.3988945908f0, 0.8954911673f0, 0.0116554042f0, 0.0909389386f0, 0.0893340926f0, 0.4953123474f0, 
            0.5784687653f0, 0.2548134842f0, 0.1776265054f0, 0.3360827756f0, 0.3734218081f0, 0.6334027459f0,
            0.8120340729f0, 0.1525260985f0, 0.0720461340f0, 0.3180398718f0, 0.3208139232f0, 0.9439490845f0, 
            0.0996337096f0, 0.3485065303f0, 0.7917933350f0, 0.8462610756f0, 0.4970552639f0, 0.9443231657f0,
            0.1459758690f0, 0.1334774229f0, 0.0101744474f0, 0.2696308750f0, 0.1566415042f0, 0.2585378565f0,
            0.3350715841f0, 0.6044406241f0, 0.0164770681f0, 0.5227222970f0, 0.3939237240f0, 0.1516453785f0,
            0.7058609147f0, 0.4322837979f0, 0.3666769617f0, 0.9135396160f0, 0.7535281491f0, 0.1228587420f0, 
            0f0 # Keep getting out of bounds errors but I don't want to actually fix them.
        ];
        push!(graph_feedback, -2f0);
        push!(graph_feedback, -2f0);
        for s in 1:42
            pos = Float32(s - 1) / (1f0 + 40f0 * first(delay_mul))
            value = if smooth_mode == 0
                dummy_waveform[floor(Int32, pos) + 1]
            else
                from = dummy_waveform[floor(Int32, pos) + 1]
                to = dummy_waveform[floor(Int32, pos) + 2]
                amount = pos % 1f0
                to * amount + from * (1f0 - amount)
            end
            push!(graph_feedback, (value * 2f0 - 1f0) * first(amplitude));
        end
    end
end
