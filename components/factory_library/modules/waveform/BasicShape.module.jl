function exec()
    waveform = function (phase::Float32, buffer_pos::Integer)
        parameter_here = parameter[%, 1, buffer_pos]
        if choice == 0
            sin(phase * pi * 2)
        elseif choice == 1
            if phase < parameter_here 
                -1f0 
            else 
                1f0 
            end
        elseif choice == 2
            if phase < parameter_here 
                phase / parameter_here * 2f0 - 1f0
            else
                (1f0 - phase) / (1f0 - parameter_here) * 2f0 - 1f0
            end
        else
            @assert choice == 3
            phase ^ exp(parameter_here * 8f0 - 4f0) * 2f0 - 1f0
        end
    end
end