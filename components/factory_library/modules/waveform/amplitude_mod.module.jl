function exec()
    waveform = function (phase::Float32, buffer_pos::Integer)
        amplitude = lerp(1.0, modulator(phase, buffer_pos), intensity[%, 1, buffer_pos])
        carrier(phase, buffer_pos) * amplitude
    end
end
