function exec()
    waveform = function(phase, buffer_pos::Integer)
        SampleType = typeof2(phase)
        sample = similar(SampleType)
        for c in channel_indices(sample)
            phase_here = phase[c]
            peak_here = peak_phase[%, buffer_pos]
            if phase_here < peak_here
                sample[c] = phase_here / peak_here * 2f0 - 1f0
            else
                sample[c] = (1f0 - phase_here) / (1f0 - peak_here) * 2f0 - 1f0
            end
        end
        # if global_update_feedback_data {
        #     DisplayWaveform(SetGraphFeedback, Waveform);
        # } 
        # end
        sample
    end
end