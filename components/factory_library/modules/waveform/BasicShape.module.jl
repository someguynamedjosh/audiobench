function exec()
    waveform = function(phase, buffer_pos::Integer)
        SampleType = typeof2(phase)
        sample = similar(SampleType)
        for c in channel_indices(sample)
            phase_here = phase[c]
            parameter_here = parameter[%, 1, buffer_pos]
            if choice == 0
                sample[c] = sin(phase_here * pi * 2)
            elseif choice == 1
                if phase_here < parameter_here 
                    sample[c] = -1f0 
                else 
                    sample[c] = 1f0 
                end
            elseif choice == 2
                if phase_here < parameter_here 
                    sample[c] = phase_here / parameter_here * 2f0 - 1f0
                else
                    sample[c] = (1f0 - phase_here) / (1f0 - parameter_here) * 2f0 - 1f0
                end
            end
        end
        sample
    end
end