mutable struct StaticData
    phase::mutable(MonoSample)
end

function static_init()
    StaticData(MonoSample(0f0))
end    

function exec()
    WaveformOutputType = w2st(waveform, MonoSample)
    AudioType = st2at(WaveformOutputType)
    oversampling = 4
    audio = similar(AudioType)
    sample = similar(WaveformOutputType)
    phase_delta = similar(MonoSample)

    for s in sample_indices(AudioType)
        sample .= 0f0
        phase_delta .= pitch[%, 1, s] ./ sample_rate ./ Float32(oversampling)
        for subsample in 1:oversampling
            sample .+= waveform(static.phase, s)
            static.phase .= (static.phase .+ phase_delta) .% 1f0
        end
        audio[:, s] .= sample .* amplitude[%, 1, s] ./ Float32(oversampling)
    end
end
