module Lib

using Main.Parameters
using Main.UnpackedDependencies.StaticArrays

const num_midi_controls = 128

const MonoSample = SArray{Tuple{1},Float32,1,1}
const StereoSample = SArray{Tuple{channels},Float32,1,channels}
const MonoBuffer = SArray{Tuple{1,buffer_length},Float32,2,buffer_length}
const StereoBuffer = SArray{Tuple{channels,buffer_length},Float32,2,channels * buffer_length}

const TriggerBuffer = SArray{Tuple{buffer_length},Bool,1,buffer_length}
const Waveform = Function

const flat_waveform = (phase, _buffer_pos::Integer) -> Float32(0) 
const ramp_up_waveform = (phase, _buffer_pos::Integer) -> @. phase * 2 - 1
const ramp_down_waveform = (phase, _buffer_pos::Integer) -> @. 1 - phase * 2

struct GlobalInput
    midi_controls::Vector{Float32}
    pitch_wheel::Float32
    bpm::Float32
    elapsed_time::Float32
    elapsed_beats::Float32
    do_update::Bool
end

struct NoteInput
    pitch::Float32
    velocity::Float32
    elapsed_time::Float32
    elapsed_beats::Float32
    start_trigger::Bool
    release_trigger::Bool
end

struct NoteContext
    global_in::GlobalInput
    note_in::NoteInput
end

mutable struct NoteOutput
    audio::StereoBuffer
end

function promote_vectorized(types::DataType...)::DataType
    Base.promote_op(Base.broadcast, typeof(+), types...)
end

function promote_typeof_vectorized(values...)::DataType
    Base.promote_op(Base.broadcast, typeof(+), broadcast(typeof, values)...)
end

# Asserts that the specified type can be promoted all the way up to 
# StereoSample without problems.
function assert_sample_type(type::DataType)
    if promote_vectorized(type, StereoSample) != StereoSample
        throw(AssertionError("$type is not a valid sample type (must be promotable to StereoSample)"))
    end
end

# Asserts that the specified type can be promoted all the way up to 
# StereoBuffer without problems.
function assert_audio_type(type::DataType)
    if promote_vectorized(type, StereoBuffer) != StereoBuffer
        throw(AssertionError("$type is not a valid audio type (must be promotable to StereoBuffer)"))
    end
end

# Asserts that the specified function has a signature that makes it usable
# as a waveform (I.E. it can accept a Mono/StereoSample and Int32 and return a valid
# sample type as described by assert_sample_type.)
function assert_waveform_func(func::Function)
    return_types = Base.return_types(func, [Float32, Int32])
    @assert length(return_types) == 1
    assert_sample_type(first(return_types))
    return_types = Base.return_types(func, [MonoSample, Int32])
    @assert length(return_types) == 1
    assert_sample_type(first(return_types))
    return_types = Base.return_types(func, [StereoSample, Int32])
    @assert length(return_types) == 1
    assert_sample_type(first(return_types))
end

# export all
# https://discourse.julialang.org/t/exportall/4970/16
for m in (@__MODULE__, Main.UnpackedDependencies.StaticArrays, Main.Parameters)
    for n in names(m; all=true)
        if Base.isidentifier(n) && n ∉ (Symbol(@__MODULE__), :eval, :include)
            @eval export $n
        end
    end
end

end # module
