function exec()
    waveform = function (phase::Float32, buffer_pos::Integer)
        offset = modulator(phase, buffer_pos) * intensity[1, buffer_pos]
        carrier((phase + offset + 1f0) % 1f0, buffer_pos)
    end
end
