module Lib

module TestParameters
    const buffer_length = 10
    const channels = 2
    export buffer_length, channels
end

if isinteractive()
    @eval Main Parameters = Main.Registry.Factory.Lib.TestParameters
    @eval Main module UnpackedDependencies using StaticArrays end
end

using Main.Parameters
using Main.UnpackedDependencies.StaticArrays

const num_midi_controls = 128

const MonoSample = SArray{Tuple{1},Float32,1,1}
const StereoSample = SArray{Tuple{channels},Float32,1,channels}
const StaticControlSignal = SArray{Tuple{1},Float32,1,1}
const ControlSignal = SArray{Tuple{buffer_length},Float32,1,buffer_length}
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
const sine_waveform = (phase, _buffer_pos::Integer) -> @. sin(phase * pi * 2f0)

function mutable(type::DataType)::DataType
    typeof(similar(type))
end

function maybe_mutable(type::DataType)
    Union{type, mutable(type)}
end

function maybe_mutable_type(type::DataType)
    Union{Type{type}, Type{mutable(type)}}
end

function typeof2(data::AbstractArray{Float32,1})
    dims = size(data)
    if dims[1] === 1
        MonoSample # Also equivalent to StaticControlSignal
    elseif dims[1] === channels
        StereoSample
    elseif dims[1] === buffer_length
        ControlSignal
    else
        @assert false "Invalid sample or control signal type"
    end
end

function typeof2(data::AbstractArray{Float32,2})
    dims = size(data)
    if dims === [1, 1]
        StaticMonoAudio
    elseif dims === [channels, 1]
        StaticStereoAudio
    elseif dims === [1, buffer_length]
        MonoAudio
    elseif dims === [channels, buffer_length]
        StereoAudio
    else
        @assert false "Invalid audio type"
    end
end

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
    audio::mutable(StereoAudio)
end

function NoteOutput()
    NoteOutput(similar(StereoAudio))
end

struct NoteContext
    global_in::GlobalInput
    note_in::NoteInput
    note_out::NoteOutput
end

# Timing modes:
# Bit 1 controls note (false) vs song (true)
# Bit 2 controls seconds (false) vs beats (true)
function get_timing(context::NoteContext, mode::Integer)::mutable(ControlSignal)
    result = similar(ControlSignal)
    song_source::Bool = mode & 0b1 === 0b1
    beat_units::Bool = mode & 0b10 === 0b10
    value::Float32 = if song_source 
        if beat_units context.global_in.elapsed_beats else context.global_in.elapsed_time end
    else 
        if beat_units context.note_in.elapsed_beats else context.note_in.elapsed_time end
    end
    per_sample::Float32 = if beat_units
        context.global_in.bpm / 60f0 / sample_rate
    else
        1f0 / sample_rate
    end
    for i in 1:buffer_length
        result[i] = value
        value += per_sample
    end
    result
end

function promote_vectorized(types::DataType...)::DataType
    Base.promote_op(Base.broadcast, typeof(+), types...)
end

function promote_typeof_vectorized(values...)::DataType
    Base.promote_op(Base.broadcast, typeof(+), broadcast(typeof, values)...)
end

# Audio type to sample type
at2st(_audio_type::Type{StaticMonoAudio})::Type{MonoSample} = MonoSample
at2st(_audio_type::Type{StaticStereoAudio})::Type{StereoSample} = StereoSample
at2st(_audio_type::Type{MonoAudio})::Type{MonoSample} = MonoSample
at2st(_audio_type::Type{StereoAudio})::Type{StereoSample} = StereoSample
at2st(_audio_type::Type{mutable(StaticMonoAudio)})::Type{MonoSample} = MonoSample
at2st(_audio_type::Type{mutable(StaticStereoAudio)})::Type{StereoSample} = StereoSample
at2st(_audio_type::Type{mutable(MonoAudio)})::Type{MonoSample} = MonoSample
at2st(_audio_type::Type{mutable(StereoAudio)})::Type{StereoSample} = StereoSample

# Sample type to audio type
st2at(_sample_type::Type{MonoSample})::Type{MonoAudio} = MonoAudio
st2at(_sample_type::Type{StereoSample})::Type{StereoAudio} = StereoAudio
st2at(_sample_type::Type{mutable(MonoSample)})::Type{MonoAudio} = MonoAudio
st2at(_sample_type::Type{mutable(StereoSample)})::Type{StereoAudio} = StereoAudio

# Sample type to static audio type
st2sat(_sample_type::Type{MonoSample})::Type{StaticMonoAudio} = StaticMonoAudio
st2sat(_sample_type::Type{StereoSample})::Type{StaticStereoAudio} = StaticStereoAudio
st2sat(_sample_type::Type{mutable(MonoSample)})::Type{StaticMonoAudio} = StaticMonoAudio
st2sat(_sample_type::Type{mutable(StereoSample)})::Type{StaticStereoAudio} = StaticStereoAudio

# Audio to control signal
a2cs(audio::maybe_mutable(StereoAudio))::ControlSignal = ControlSignal(sum(audio; dims=1) ./ 2)
a2cs(audio::maybe_mutable(MonoAudio))::ControlSignal = ControlSignal(audio)
a2cs(audio::maybe_mutable(StaticStereoAudio))::StaticControlSignal = StaticControlSignal((audio[1] + audio[2]) / 2)
a2cs(audio::maybe_mutable(StaticMonoAudio))::StaticControlSignal = StaticControlSignal(audio)

function assert_is_sample_type(_type::Type{MonoSample}) end
function assert_is_sample_type(_type::Type{StereoSample}) end
function assert_is_sample_type(type) 
    throw(AssertionError("$type is not a valid sample type."))
end

function assert_is_control_signal_type(_type::Type{StaticControlSignal}) end
function assert_is_control_signal_type(_type::Type{ControlSignal}) end
function assert_is_control_signal_type(type) 
    throw(AssertionError("$type is not a valid control signal type."))
end

function assert_is_audio_type(_type::Type{StaticMonoAudio}) end
function assert_is_audio_type(_type::Type{StaticStereoAudio}) end
function assert_is_audio_type(_type::Type{MonoAudio}) end
function assert_is_audio_type(_type::Type{StereoAudio}) end
function assert_is_audio_type(type) 
    throw(AssertionError("$type is not a valid audio type."))
end

function assert_is_trigger_type(_type::Type{StaticTrigger}) end
function assert_is_trigger_type(_type::Type{Trigger}) end
function assert_is_trigger_type(type) 
    throw(AssertionError("$type is not a valid trigger type."))
end

# Waveform to sample type
function w2st(waveform::Waveform, _phase_type::maybe_mutable_type(MonoSample))::Union{maybe_mutable_type(MonoSample), maybe_mutable_type(StereoSample)}
    Base.promote_op(waveform, MonoSample, Int32)
end

function w2st(waveform::Waveform, _phase_type::maybe_mutable_type(StereoSample))::Union{maybe_mutable_type(MonoSample), maybe_mutable_type(StereoSample)}
    Base.promote_op(waveform, StereoSample, Int32)
end

# Asserts that the specified function has a signature that makes it usable
# as a waveform (I.E. it can accept a Mono/StereoSample and Int32 and return a valid
# sample type as described by assert_is_sample_type.)
function assert_is_waveform(func::Function)
    assert_is_sample_type(w2st(func, MonoSample))
    assert_is_sample_type(w2st(func, StereoSample))
end

# Allows indexing smaller buffers as if they were their full-sized counterparts.
Base.getindex(from::maybe_mutable(StereoAudio), _::typeof(%), channelidx::Integer, sampleidx::Integer)::Float32 = from[channelidx, sampleidx]
Base.getindex(from::maybe_mutable(MonoAudio), _::typeof(%), channelidx::Integer, sampleidx::Integer)::Float32 = from[1, sampleidx]
Base.getindex(from::maybe_mutable(StaticStereoAudio), _::typeof(%), channelidx::Integer, sampleidx::Integer)::Float32 = from[channelidx, 1]
Base.getindex(from::maybe_mutable(StaticMonoAudio), _::typeof(%), channelidx::Integer, sampleidx::Integer)::Float32 = from[1, 1]

Base.getindex(from::maybe_mutable(StereoSample), _::typeof(%), channelidx::Integer)::Float32 = from[channelidx]
Base.getindex(from::maybe_mutable(MonoSample), _::typeof(%), channelidx::Integer)::Float32 = from[1]

Base.getindex(from::maybe_mutable(ControlSignal), _::typeof(%), sampleidx::Integer)::Float32 = from[sampleidx]
# Ambiguous with MonoSample, but same functionality.
# Base.getindex(from::maybe_mutable(StaticControlSignal), _::typeof(%), sampleidx::Integer)::Float32 = from[1]

Base.getindex(from::maybe_mutable(Trigger), _::typeof(%), sampleidx::Integer)::Bool = from[sampleidx]
Base.getindex(from::maybe_mutable(StaticTrigger), _::typeof(%), sampleidx::Integer)::Bool = from[1]

# Allows accessing static data as a smaller data type. Cannot view small data as a bigger type.
viewas(data::Union{maybe_mutable(MonoSample), maybe_mutable(StereoSample)}, type::maybe_mutable_type(MonoSample)) = @view data[1:1]
viewas(data::maybe_mutable(StereoSample), type::maybe_mutable_type(StereoSample)) = @view data[:]

viewas(data::Union{maybe_mutable(StaticControlSignal), maybe_mutable(ControlSignal)}, type::maybe_mutable_type(StaticControlSignal)) = @view data[1:1]
viewas(data::maybe_mutable(ControlSignal), type::maybe_mutable_type(ControlSignal)) = @view data[:]

viewas(data::Union{maybe_mutable(StaticTrigger), maybe_mutable(Trigger)}, type::maybe_mutable_type(StaticTrigger)) = @view data[1:1]
viewas(data::maybe_mutable(Trigger), type::maybe_mutable_type(Trigger)) = @view data[:]

viewas(data::Union{maybe_mutable(StaticMonoAudio), maybe_mutable(StaticStereoAudio), maybe_mutable(MonoAudio), maybe_mutable(StereoAudio)}, type::Type{StaticMonoAudio}) = @view data[1:1, 1:1]
viewas(data::Union{maybe_mutable(StaticStereoAudio), maybe_mutable(StereoAudio)}, type::Type{StaticStereoAudio}) = @view data[:, 1:1]
viewas(data::Union{maybe_mutable(MonoAudio), maybe_mutable(StereoAudio)}, type::Type{MonoAudio}) = @view data[1:1, :]
viewas(data::maybe_mutable(StereoAudio), type::Type{StereoAudio}) = @view data[:, :]

# Allows manually iterating over audio.
sample_indices(_buf::SArray{Tuple{C, S}, Float32, 2, N}) where {C, S, N} = Base.OneTo(S)
sample_indices(_buf::MArray{Tuple{C, S}, Float32, 2, N}) where {C, S, N} = Base.OneTo(S)
sample_indices(_buf::SizedArray{Tuple{C, S}, Float32, 2, N}) where {C, S, N} = Base.OneTo(S)
sample_indices(_buf::Type{SArray{Tuple{C, S}, Float32, 2, N}}) where {C, S, N} = Base.OneTo(S)
sample_indices(_buf::Type{MArray{Tuple{C, S}, Float32, 2, N}}) where {C, S, N} = Base.OneTo(S)
sample_indices(_buf::Type{SizedArray{Tuple{C, S}, Float32, 2, N}}) where {C, S, N} = Base.OneTo(S)
channel_indices(_buf::SArray{Tuple{C, S}, Float32, 2, N}) where {C, S, N} = Base.OneTo(C)
channel_indices(_buf::MArray{Tuple{C, S}, Float32, 2, N}) where {C, S, N} = Base.OneTo(C)
channel_indices(_buf::SizedArray{Tuple{C, S}, Float32, 2, N}) where {C, S, N} = Base.OneTo(C)
channel_indices(_buf::Type{SArray{Tuple{C, S}, Float32, 2, N}}) where {C, S, N} = Base.OneTo(C)
channel_indices(_buf::Type{MArray{Tuple{C, S}, Float32, 2, N}}) where {C, S, N} = Base.OneTo(C)
channel_indices(_buf::Type{SizedArray{Tuple{C, S}, Float32, 2, N}}) where {C, S, N} = Base.OneTo(C)
# ...samples.
channel_indices(_buf::SArray{Tuple{C}, Float32, 1, N}) where {C, N} = Base.OneTo(C)
channel_indices(_buf::MArray{Tuple{C}, Float32, 1, N}) where {C, N} = Base.OneTo(C)
channel_indices(_buf::SizedArray{Tuple{C}, Float32, 1, N}) where {C, N} = Base.OneTo(C)
channel_indices(_buf::Type{SArray{Tuple{C}, Float32, 1, N}}) where {C, N} = Base.OneTo(C)
channel_indices(_buf::Type{MArray{Tuple{C}, Float32, 1, N}}) where {C, N} = Base.OneTo(C)
channel_indices(_buf::Type{SizedArray{Tuple{C}, Float32, 1, N}}) where {C, N} = Base.OneTo(C)
# ...control signals.
sample_indices(_buf::SArray{Tuple{S}, Float32, 1, N}) where {S, N} = Base.OneTo(S)
sample_indices(_buf::MArray{Tuple{S}, Float32, 1, N}) where {S, N} = Base.OneTo(S)
sample_indices(_buf::SizedArray{Tuple{S}, Float32, 1, N}) where {S, N} = Base.OneTo(S)
sample_indices(_buf::Type{SArray{Tuple{S}, Float32, 1, N}}) where {S, N} = Base.OneTo(S)
sample_indices(_buf::Type{MArray{Tuple{S}, Float32, 1, N}}) where {S, N} = Base.OneTo(S)
sample_indices(_buf::Type{SizedArray{Tuple{S}, Float32, 1, N}}) where {S, N} = Base.OneTo(S)
# ...triggers.
sample_indices(_buf::SArray{Tuple{S}, Bool, 1, N}) where {S, N} = Base.OneTo(S)
sample_indices(_buf::MArray{Tuple{S}, Bool, 1, N}) where {S, N} = Base.OneTo(S)
sample_indices(_buf::SizedArray{Tuple{S}, Float32, 1, N}) where {S, N} = Base.OneTo(S)
sample_indices(_buf::Type{SArray{Tuple{S}, Bool, 1, N}}) where {S, N} = Base.OneTo(S)
sample_indices(_buf::Type{MArray{Tuple{S}, Bool, 1, N}}) where {S, N} = Base.OneTo(S)
sample_indices(_buf::Type{SizedArray{Tuple{S}, Float32, 1, N}}) where {S, N} = Base.OneTo(S)

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
