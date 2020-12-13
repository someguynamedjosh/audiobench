module Lib

module TestParameters
    const buffer_length = 10
    const channels = 2
    export buffer_length, channels
end

if isinteractive()
    @eval Main Parameters = Main.Lib.TestParameters
    @eval Main module UnpackedDependencies using StaticArrays end
end

using Main.Parameters
using Main.UnpackedDependencies.StaticArrays

const num_midi_controls = 128

const MonoSample = SArray{Tuple{1},Float32,1,1}
const StereoSample = SArray{Tuple{channels},Float32,1,channels}
const StaticMonoAudio = SArray{Tuple{1,1},Float32,2,1}
const StaticStereoAudio = SArray{Tuple{channels,1},Float32,2,channels}
const MonoAudio = SArray{Tuple{1,buffer_length},Float32,2,buffer_length}
const StereoAudio = SArray{Tuple{channels,buffer_length},Float32,2,channels * buffer_length}

const StaticTrigger = SArray{Tuple{1},Bool,1,1}
const Trigger = SArray{Tuple{buffer_length},Bool,1,buffer_length}
const Waveform = Function

const flat_waveform = (phase, _buffer_pos::Integer) -> mono_sample(0f0) 
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

mutable struct NoteOutput
    audio::StereoAudio
end

struct NoteContext
    global_in::GlobalInput
    note_in::NoteInput
    note_out::NoteOutput
end

function promote_vectorized(types::DataType...)::DataType
    Base.promote_op(Base.broadcast, typeof(+), types...)
end

function promote_typeof_vectorized(values...)::DataType
    Base.promote_op(Base.broadcast, typeof(+), broadcast(typeof, values)...)
end

function mutable(type::DataType)::DataType
    typeof(similar(type))
end

function maybe_mutable(type::DataType)
    Union{type, mutable(type)}
end

function assert_sample_type(_type::Type{MonoSample}) end
function assert_sample_type(_type::Type{StereoSample}) end
function assert_sample_type(type) 
    throw(AssertionError("$type is not a valid sample type."))
end

function MonoSample(value::Float32)::MonoSample
    return SA_F32[value]
end

function StereoSample(left::Float32, right::Float32)::StereoSample
    @assert channels == 2
    return SA_F32[left, right]
end

function assert_audio_type(_type::Type{StaticMonoAudio}) end
function assert_audio_type(_type::Type{StaticStereoAudio}) end
function assert_audio_type(_type::Type{MonoAudio}) end
function assert_audio_type(_type::Type{StereoAudio}) end
function assert_audio_type(type) 
    throw(AssertionError("$type is not a valid sample type."))
end

at2st(_audio_type::Type{StaticMonoAudio})::Type{MonoSample} = MonoSample
at2st(_audio_type::Type{StaticStereoAudio})::Type{StereoSample} = StereoSample
at2st(_audio_type::Type{MonoAudio})::Type{MonoSample} = MonoSample
at2st(_audio_type::Type{StereoAudio})::Type{StereoSample} = StereoSample
at2st(_audio_type::Type{mutable(StaticMonoAudio)})::Type{MonoSample} = MonoSample
at2st(_audio_type::Type{mutable(StaticStereoAudio)})::Type{StereoSample} = StereoSample
at2st(_audio_type::Type{mutable(MonoAudio)})::Type{MonoSample} = MonoSample
at2st(_audio_type::Type{mutable(StereoAudio)})::Type{StereoSample} = StereoSample

st2at(_sample_type::Type{MonoSample})::Type{MonoAudio} = MonoAudio
st2at(_sample_type::Type{StereoSample})::Type{StereoAudio} = StereoAudio
st2at(_sample_type::Type{mutable(MonoSample)})::Type{MonoAudio} = MonoAudio
st2at(_sample_type::Type{mutable(StereoSample)})::Type{StereoAudio} = StereoAudio

st2sat(_sample_type::Type{MonoSample})::Type{StaticMonoAudio} = StaticMonoAudio
st2sat(_sample_type::Type{StereoSample})::Type{StaticStereoAudio} = StaticStereoAudio
st2sat(_sample_type::Type{mutable(MonoSample)})::Type{StaticMonoAudio} = StaticMonoAudio
st2sat(_sample_type::Type{mutable(StereoSample)})::Type{StaticStereoAudio} = StaticStereoAudio

function StaticMonoAudio(value::Float32)::StaticMonoAudio
    return SA_F32[value]
end

function StaticStereoAudio(left::Float32, right::Float32)::StaticStereoAudio
    @assert channels == 2
    return SA_F32[left; right]
end

function assert_trigger_type(_type::Type{StaticTrigger}) end
function assert_trigger_type(_type::Type{Trigger}) end
function assert_trigger_type(type) 
    throw(AssertionError("$type is not a valid trigger type."))
end

function w2st(waveform::Waveform, _phase_type::Type{MonoSample})::Union{Type{MonoSample}, Type{StereoSample}}
    Base.promote_op(waveform, MonoSample, Int32)
end

function w2st(waveform::Waveform, _phase_type::Type{StereoSample})::Union{Type{MonoSample}, Type{StereoSample}}
    Base.promote_op(waveform, StereoSample, Int32)
end

# Asserts that the specified function has a signature that makes it usable
# as a waveform (I.E. it can accept a Mono/StereoSample and Int32 and return a valid
# sample type as described by assert_sample_type.)
function assert_waveform_func(func::Function)
    assert_sample_type(get_waveform_output_type(func, MonoSample))
    assert_sample_type(get_waveform_output_type(func, StereoSample))
end

# Allows indexing smaller buffers as if they were their full-sized counterparts.
Base.getindex(from::maybe_mutable(MonoAudio), channelidx::Int64, sampleidx::Int64)::Float32 = from[sampleidx]
Base.getindex(from::maybe_mutable(StaticMonoAudio), channelidx::Int64, sampleidx::Int64)::Float32 = from[1]
Base.getindex(from::maybe_mutable(StaticStereoAudio), channelidx::Int64, sampleidx::Int64)::Float32 = from[channelidx]
Base.getindex(from::maybe_mutable(StaticTrigger), sampleidx::Int64)::Bool = from[1]

# Allows accessing static data as a smaller data type. Cannot view small data as a bigger type.
viewas(data::Union{maybe_mutable(MonoSample), maybe_mutable(StereoSample)}, type::Type{MonoSample}) = @view data[1]
viewas(data::maybe_mutable(StereoSample), type::Type{StereoSample}) = @view data[:]
viewas(data::Union{mutable(MonoSample), mutable(StereoSample)}, type::Type{mutable(MonoSample)}) = @view data[1]
viewas(data::mutable(StereoSample), type::Type{mutable(StereoSample)}) = @view data[:]

viewas(data::Union{maybe_mutable(StaticTrigger), maybe_mutable(Trigger)}, type::Type{StaticTrigger}) = @view data[1]
viewas(data::maybe_mutable(Trigger), type::Type{Trigger}) = @view data[:]
viewas(data::Union{mutable(StaticTrigger), mutable(Trigger)}, type::Type{mutable(StaticTrigger)}) = @view data[1]
viewas(data::mutable(Trigger), type::Type{mutable(Trigger)}) = @view data[:]

viewas(data::Union{maybe_mutable(StaticMonoAudio), maybe_mutable(StaticStereoAudio), maybe_mutable(MonoAudio), maybe_mutable(StereoAudio)}, type::Type{StaticMonoAudio}) = @view data[1, 1]
viewas(data::Union{maybe_mutable(StaticStereoAudio), maybe_mutable(StereoAudio)}, type::Type{StaticStereoAudio}) = @view data[:, 1]
viewas(data::Union{maybe_mutable(MonoAudio), maybe_mutable(StereoAudio)}, type::Type{MonoAudio}) = @view data[1, :]
viewas(data::maybe_mutable(StereoAudio), type::Type{StereoAudio}) = @view data[:, :]
viewas(data::Union{mutable(StaticMonoAudio), mutable(StaticStereoAudio), mutable(MonoAudio), mutable(StereoAudio)}, type::Type{mutable(StaticMonoAudio)}) = @view data[1, 1]
viewas(data::Union{mutable(StaticStereoAudio), mutable(StereoAudio)}, type::Type{mutable(StaticStereoAudio)}) = @view data[:, 1]
viewas(data::Union{mutable(MonoAudio), mutable(StereoAudio)}, type::Type{mutable(MonoAudio)}) = @view data[1, :]
viewas(data::mutable(StereoAudio), type::Type{mutable(StereoAudio)}) = @view data[:, :]

# Allows manually iterating over buffers.
sample_indices(_buf::SArray{Tuple{C, S}, Float32, 2, N}) where {C, S, N} = Base.OneTo(S)
sample_indices(_buf::MArray{Tuple{C, S}, Float32, 2, N}) where {C, S, N} = Base.OneTo(S)
sample_indices(_buf::Type{SArray{Tuple{C, S}, Float32, 2, N}}) where {C, S, N} = Base.OneTo(S)
sample_indices(_buf::Type{MArray{Tuple{C, S}, Float32, 2, N}}) where {C, S, N} = Base.OneTo(S)
channel_indices(_buf::SArray{Tuple{C, S}, Float32, 2, N}) where {C, S, N} = Base.OneTo(C)
channel_indices(_buf::MArray{Tuple{C, S}, Float32, 2, N}) where {C, S, N} = Base.OneTo(C)
channel_indices(_buf::Type{SArray{Tuple{C, S}, Float32, 2, N}}) where {C, S, N} = Base.OneTo(C)
channel_indices(_buf::Type{MArray{Tuple{C, S}, Float32, 2, N}}) where {C, S, N} = Base.OneTo(C)
# For samples.
channel_indices(_buf::SArray{Tuple{C}, Float32, 1, N}) where {C, N} = Base.OneTo(C)
channel_indices(_buf::MArray{Tuple{C}, Float32, 1, N}) where {C, N} = Base.OneTo(C)
channel_indices(_buf::Type{SArray{Tuple{C}, Float32, 1, N}}) where {C, N} = Base.OneTo(C)
channel_indices(_buf::Type{MArray{Tuple{C}, Float32, 1, N}}) where {C, N} = Base.OneTo(C)
# For trigger buffers.
sample_indices(_buf::SArray{Tuple{S}, Bool, 1, N}) where {S, N} = Base.OneTo(S)
sample_indices(_buf::MArray{Tuple{S}, Bool, 1, N}) where {S, N} = Base.OneTo(S)
sample_indices(_buf::Type{SArray{Tuple{S}, Bool, 1, N}}) where {S, N} = Base.OneTo(S)
sample_indices(_buf::Type{MArray{Tuple{S}, Bool, 1, N}}) where {S, N} = Base.OneTo(S)

# export all
# https://discourse.julialang.org/t/exportall/4970/16
for m in (@__MODULE__, Main.UnpackedDependencies.StaticArrays, Main.Parameters)
    for n in names(m; all=true)
        if Base.isidentifier(n) && n ∉ (Symbol(@__MODULE__), :eval, :include)
            @eval export $n
        end
    end
end

end # module Lib
