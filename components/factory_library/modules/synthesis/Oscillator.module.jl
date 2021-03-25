mutable struct StaticData
    phase::Float32
end

function static_init()
    StaticData(0f0)
end    

function exec()
    oversampling = 4
    audio = similar(MonoAudio)

    for s in sample_indices(MonoAudio)
        sample = 0f0
        phase_delta = pitch[1, s] / sample_rate / Float32(oversampling)
        for subsample in 1:oversampling
            sample += waveform(static.phase, s)
            static.phase = (static.phase + phase_delta) % 1f0
        end
        audio[1, s] = sample * amplitude[1, s] / Float32(oversampling)
    end
end
