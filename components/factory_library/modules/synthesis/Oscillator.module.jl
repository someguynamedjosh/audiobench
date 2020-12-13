mutable struct StaticData
    phase::StereoSample
end

function static_init()
    StaticData(StereoSample(0f0, 0f0))
end    

function exec()
    PhaseType = at2st(typeof(pitch))
    WaveformOutputType = w2st(waveform, PhaseType)
    AudioType = promote_vectorized(st2at(WaveformOutputType), typeof(amplitude))
    oversampling = 4
    audio = similar(AudioType)
    sample = similar(WaveformOutputType)
    phase = viewas(static.phase, mutable(PhaseType))
    phase_delta = similar(phase)

    @views for s in sample_indices(AudioType)
        sample .= 0f0
        phase_delta .= pitch[:, s] ./ sample_rate ./ Float32(oversampling)
        for subsample in 1:oversampling
            sample .+= waveform(s, phase)
            phase .= (phase .+ phase_delta) .% 1f0
        end
        audio[:, s] .= sample .* amplitude[:, s] ./ Float32(oversampling)
    end
end
