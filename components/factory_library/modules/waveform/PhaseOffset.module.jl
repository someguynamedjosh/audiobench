function exec()
    waveform = function (phase::Float32, buffer_pos::Integer)
        carrier((phase + offset[%, 1, buffer_pos] + 1f0) % 1f0, buffer_pos)
    end
end
