mutable struct StaticData
    phase::mutable(StereoSample)
end

function static_init()
    phase = similar(StereoSample)
    phase .= 0f0
    StaticData(phase)
end

function exec()
    oversampling = 4
    SampleType = at2st(typeof2(fm_signal))
    phase = viewas(static.phase, SampleType)
    phase_delta = similar(phase)
    sample = similar(phase)
    pitch_here = similar(phase)
    audio = similar(st2at(SampleType))

    for s in sample_indices(MonoAudio)
        @. sample = 0f0
        @. pitch_here = pitch[%, 1, s] * (fm_signal[%, :, s] * fm_strength[%, 1, s] + 1f0)
        @. phase_delta = pitch_here / sample_rate / Float32(oversampling) + 1f0
        for subsample in 1:oversampling
            @. sample += waveform(phase, (s,))
            @. phase = (phase + phase_delta) % 1f0
        end
        @. audio[:, s] = sample * amplitude[%, 1, s] / Float32(oversampling)
    end
end
