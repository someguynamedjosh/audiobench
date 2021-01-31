function exec()
    out_wave = function (phase::Float32, buffer_pos::Integer)
        base_wave((phase * harmonic) % 1f0, buffer_pos)
    end
end
