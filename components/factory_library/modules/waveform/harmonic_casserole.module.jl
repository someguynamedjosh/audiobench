function exec()
    out_wave = function (phase::Float32, buffer_pos::Integer)
        result = 0f0

        this_phase = (phase * 1 + 1f0 + phase1[%, 1, buffer_pos]) % 1f0
        result += base_wave(this_phase, buffer_pos) * amp1[%, 1, buffer_pos]
        this_phase = (phase * 2 + 1f0 + phase2[%, 1, buffer_pos]) % 1f0
        result += base_wave(this_phase, buffer_pos) * amp2[%, 1, buffer_pos]
        this_phase = (phase * 3 + 1f0 + phase3[%, 1, buffer_pos]) % 1f0
        result += base_wave(this_phase, buffer_pos) * amp3[%, 1, buffer_pos]
        this_phase = (phase * 4 + 1f0 + phase4[%, 1, buffer_pos]) % 1f0
        result += base_wave(this_phase, buffer_pos) * amp4[%, 1, buffer_pos]
        this_phase = (phase * 5 + 1f0 + phase5[%, 1, buffer_pos]) % 1f0
        result += base_wave(this_phase, buffer_pos) * amp5[%, 1, buffer_pos]
        this_phase = (phase * 6 + 1f0 + phase6[%, 1, buffer_pos]) % 1f0
        result += base_wave(this_phase, buffer_pos) * amp6[%, 1, buffer_pos]
        this_phase = (phase * 7 + 1f0 + phase7[%, 1, buffer_pos]) % 1f0
        result += base_wave(this_phase, buffer_pos) * amp7[%, 1, buffer_pos]
        this_phase = (phase * 8 + 1f0 + phase8[%, 1, buffer_pos]) % 1f0
        result += base_wave(this_phase, buffer_pos) * amp8[%, 1, buffer_pos]

        result * post_amp[%, 1, buffer_pos]
    end
end
