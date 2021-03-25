module Lib

module TestParameters
    const buffer_length = 10
    const channels = 2
    export buffer_length, channels
end

if isinteractive()
    @eval Main Parameters = Main.Registry.Factory.Lib.TestParameters
end

using Main.Parameters

struct FixedArray{T,D,ND,NI} <: AbstractArray{T,ND}
    data::Array{T,ND}
end
FixedArray{T,D,ND,NI}(value::T) where {T,D,ND,NI} =
    FixedArray{T,D,ND,NI}(repeat([value], D.parameters...))

import Base.similar
@generated function similar(array::Type{FixedArray{T,D,ND,NI}})::FixedArray{T,D,ND,NI} where {T,D,ND,NI}
    quote
        FixedArray{$T,$D,$ND,$NI}(similar(Array{$T,$ND}, $(tuple(D.parameters...))))
    end
end
@generated function similar(array::FixedArray{T,D,ND,NI})::FixedArray{T,D,ND,NI} where {T,D,ND,NI}
    quote
        similar($array)
    end
end

import Base.getindex
@generated function getindex(array::FixedArray{T,D,ND,NI}, index::Vararg{Integer,ND})::T where {T,D,ND,NI}
    real_indices = [
        if dim_value == 1
            quote 1 end
        else
            quote index[$dim_index] end
        end
        for (dim_index, dim_value) ∈ enumerate(D.parameters)
    ]
    quote
        getindex(array.data, $(real_indices...))
    end
end
function getindex(array::FixedArray{T,D,ND,NI}, index::CartesianIndex{ND})::T where {T,D,ND,NI}
    getindex(array, Tuple(index)...)
end

import Base.setindex!
@generated function setindex!(array::FixedArray{T,D,ND,NI}, value::I, index::Vararg{Integer,ND}) where {T,D,ND,NI,I}
    real_indices = [
        if dim_value == 1
            quote 1 end
        else
            quote index[$dim_index] end
        end
        for (dim_index, dim_value) ∈ enumerate(D.parameters)
    ]
    quote
        setindex!(array.data, value, $(real_indices...))
    end
end
function setindex!(array::FixedArray{T,D,ND,NI}, value::I, index::CartesianIndex{ND}) where {T,D,ND,NI,I}
    setindex!(array, value, Tuple(index)...)
end

Base.Broadcast.broadcastable(array::FixedArray{T,D,ND,NI}) where {T,D,ND,NI} = array
@generated Base.Broadcast.ndims(array::Type{FixedArray{T,D,ND,NI}}) where {T,D,ND,NI} = ND
@generated Base.Broadcast.size(array::FixedArray{T,D,ND,NI}) where {T,D,ND,NI} = :($(tuple(D.parameters...)))
@generated Base.Broadcast.size(array::FixedArray{T,D,ND,NI}, dim::Integer) where {T,D,ND,NI} = quote $(D.parameters)[dim] end
# Adapted from StaticArrays: https://github.com/JuliaArrays/StaticArrays.jl/blob/5cb521a5ff2b7a1625ecad03c1ac756dacabb346/src/broadcast.jl
import Base.Broadcast:
BroadcastStyle, AbstractArrayStyle, Broadcasted, DefaultArrayStyle, materialize!
struct FixedArrayStyle{N} <: AbstractArrayStyle{N} end
FixedArrayStyle{M}(::Val{N}) where {M,N} = FixedArrayStyle{N}()
BroadcastStyle(::Type{FixedArray{T, D, ND, NI}}) where {T,D,ND,NI} = FixedArrayStyle{ND}()
BroadcastStyle(::FixedArrayStyle{N}, ::FixedArrayStyle{M}) where {M,N} =
    FixedArrayStyle{max(M, N)}()
BroadcastStyle(fix::FixedArrayStyle{M}, ::DefaultArrayStyle{N}) where {M,N} =
    if N == 0 fix else DefaultArrayStyle(Val(max(M, N))) end
BroadcastStyle(::DefaultArrayStyle{N}, fix::FixedArrayStyle{M}) where {M,N} =
    if N == 0 fix else DefaultArrayStyle(Val(max(M, N))) end
scalar_getindex(x) = x
scalar_getindex(x::Ref) = x[]
# copy overload
@inline function Base.copy(B::Broadcasted{FixedArrayStyle{M}}) where M
    flat = Broadcast.flatten(B); as = flat.args; f = flat.f
    argsizes = map(Base.Broadcast.size, as)
    destsize = Base.Broadcast.broadcast_shape(argsizes...)
    _broadcast(f, Val(destsize), Val(argsizes), as...)
end
@inline function Base.copyto!(dest::FixedArray{T,D,M,NI}, B::Broadcasted{FixedArrayStyle{M}}) where {M,T,D,NI}
    flat = Broadcast.flatten(B); as = flat.args; f = flat.f
    argsizes = map(Base.Broadcast.size, as)
    destsize = Base.Broadcast.broadcast_shape(Base.Broadcast.size(dest), argsizes...)
    _broadcast!(f, Val(destsize), dest, Val(argsizes), as...)
end
@generated function _broadcast(f, _newsize::Val{newsize}, s::Val{sizes}, a...) where {newsize, sizes}
    first_staticarray = a[findfirst(ai -> ai <: FixedArray, a)]

    if prod(newsize) == 0
        @assert false "We don't handle this yet."
    end

    first_expr_values = [
        begin
            if !(a[i] <: AbstractArray || a[i] <: Tuple)
                :(scalar_getindex(a[$i]))
            else
                all_ones = [:(1) for _ = 1:length(sizes[i])]
                :(a[$i][$(all_ones...)])
            end
        end
        for i = 1:length(sizes)
    ]
    first_expr = :(f($(first_expr_values...)))

    return quote
        @inbounds result = similar(FixedArray{typeof($first_expr),Tuple{$(newsize...)},$(length(newsize)),$(prod(newsize))})
        _broadcast!(f, _newsize, result, s, a...)
        @inbounds return result
    end
end
@generated function _broadcast!(f, ::Val{newsize}, dest, s::Val{sizes}, a...) where {newsize, sizes}
    first_staticarray = a[findfirst(ai -> ai <: FixedArray, a)]

    if prod(newsize) == 0
        @assert false "We don't handle this yet."
    end

    index_symbols = [Symbol(:idx, dimidx) for dimidx = 1:length(newsize)]
    exprs_vals = [
        begin
            if !(a[i] <: AbstractArray || a[i] <: Tuple)
                :(scalar_getindex(a[$i]))
            else
                :(a[$i][$([index_symbols[i] for i = 1:length(sizes[i])]...)])
            end
        end
        for i = 1:length(sizes)
    ]
    compute_expr = :(dest[$(index_symbols...)] = f($(exprs_vals...)))
    for (dimidx, dim) in enumerate(newsize)
        compute_expr = quote
            for $(index_symbols[dimidx]) = 1:$(newsize[dimidx])
                $compute_expr
            end
        end
    end

    return quote
        @inbounds $compute_expr
        @inbounds return dest
    end
end

@generated function fixed_array_type(typ::Type, dimensions::Type)::Type{FixedArray}
    eltype = typ.parameters[1]
    dims = [convert(Int64, x) for x ∈ dimensions.parameters[1].parameters]
    dims = Tuple{dims...}
    ndims = length(dims.parameters)
    nitems = reduce(*, dims.parameters)
    quote
        FixedArray{$eltype, $dims, $ndims, $nitems}
    end
end

const num_midi_controls = 128
const default_graph_resolution = 42

const MonoSample = fixed_array_type(Float32, Tuple{1})
const StereoSample = fixed_array_type(Float32, Tuple{2})

const StaticMonoAudio = fixed_array_type(Float32, Tuple{1, 1})
const StaticStereoAudio = fixed_array_type(Float32, Tuple{channels, 1})
const MonoAudio = fixed_array_type(Float32, Tuple{1, buffer_length})
const StereoAudio = fixed_array_type(Float32, Tuple{channels, buffer_length})
const StaticControlSignal = StaticMonoAudio
const ControlSignal = MonoAudio

const StaticTrigger = fixed_array_type(Bool, Tuple{1, 1})
const Trigger = fixed_array_type(Bool, Tuple{1, buffer_length})
const Waveform = Function

const flat_waveform = (phase::Float32, _buffer_pos::Integer) -> 0f0
const ramp_up_waveform = (phase::Float32, _buffer_pos::Integer) -> phase * 2 - 1
const ramp_down_waveform = (phase::Float32, _buffer_pos::Integer) -> 1 - phase * 2
const sine_waveform = (phase::Float32, _buffer_pos::Integer) -> sin(phase * pi * 2f0)

struct GlobalInput
    midi_controls::Vector{Float32}
    pitch_wheel::Float32
    bpm::Float32
    elapsed_time::Float32
    elapsed_beats::Float32
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

function NoteOutput()
    NoteOutput(similar(StereoAudio))
end

struct NoteContext
    global_in::GlobalInput
    note_in::NoteInput
    note_out::NoteOutput
end

function timing_mode_source_is_global(mode::Integer)::Bool
    mode & 0b1 == 0b1
end

function timing_mode_source_is_note(mode::Integer)::Bool
    mode & 0b1 == 0b0
end

function timing_mode_unit_is_beats(mode::Integer)::Bool
    mode & 0b10 == 0b10
end

function timing_mode_unit_is_seconds(mode::Integer)::Bool
    mode & 0b10 == 0b00
end

# Timing modes:
# Bit 1 controls note (false) vs song (true)
# Bit 2 controls seconds (false) vs beats (true)
function get_timing(context::NoteContext, mode::Integer)::ControlSignal
    result = similar(ControlSignal)
    global_source::Bool = timing_mode_source_is_global(mode)
    beat_units::Bool = timing_mode_unit_is_beats(mode)
    value::Float32 = if global_source 
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
        result[1, i] = value
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

# Sample type to audio type
st2at(_sample_type::Type{MonoSample})::Type{MonoAudio} = MonoAudio
st2at(_sample_type::Type{StereoSample})::Type{StereoAudio} = StereoAudio

# Sample type to static audio type
st2sat(_sample_type::Type{MonoSample})::Type{StaticMonoAudio} = StaticMonoAudio
st2sat(_sample_type::Type{StereoSample})::Type{StaticStereoAudio} = StaticStereoAudio

# Audio to control signal
a2cs(audio::StereoAudio)::ControlSignal = ControlSignal(sum(audio; dims=1) ./ 2)
a2cs(audio::MonoAudio)::ControlSignal = ControlSignal(audio)
a2cs(audio::StaticStereoAudio)::StaticControlSignal = StaticControlSignal((audio[1] + audio[2]) / 2)
a2cs(audio::StaticMonoAudio)::StaticControlSignal = StaticControlSignal(audio)

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

# Asserts that the specified function has a signature that makes it usable
# as a waveform (I.E. it can accept a Float32 and Int32 and return a Float32
function assert_is_waveform(func::Function)
    result = func(0f0, Int32(1))
    typ = typeof(result)
    if typ != Float32
        @assert false "Waveform produces incorrect result type $typ"
    end
end

# Allows accessing static data as a smaller data type. Cannot view small data as a bigger type.
viewas(data::Union{(MonoSample), (StereoSample)}, type::(MonoSample)) = @view data[1:1]
viewas(data::(StereoSample), type::(StereoSample)) = @view data[:]

viewas(data::Union{(StaticTrigger), (Trigger)}, type::(StaticTrigger)) = @view data[1:1]
viewas(data::(Trigger), type::(Trigger)) = @view data[:]

viewas(data::Union{(StaticMonoAudio), (StaticStereoAudio), (MonoAudio), (StereoAudio)}, type::Type{StaticMonoAudio}) = @view data[1:1, 1:1]
viewas(data::Union{(StaticStereoAudio), (StereoAudio)}, type::Type{StaticStereoAudio}) = @view data[:, 1:1]
viewas(data::Union{(MonoAudio), (StereoAudio)}, type::Type{MonoAudio}) = @view data[1:1, :]
viewas(data::(StereoAudio), type::Type{StereoAudio}) = @view data[:, :]

# Allows manually iterating over audio.
sample_indices(_buf::FixedArray{Float32, Tuple{C, S}, 2, N}) where {C, S, N} = Base.OneTo(S)
sample_indices(_buf::Type{FixedArray{Float32, Tuple{C, S}, 2, N}}) where {C, S, N} = Base.OneTo(S)
channel_indices(_buf::FixedArray{Float32, Tuple{C, S}, 2, N}) where {C, S, N} = Base.OneTo(C)
channel_indices(_buf::Type{FixedArray{Float32, Tuple{C, S}, 2, N}}) where {C, S, N} = Base.OneTo(C)
# ...samples.
channel_indices(_buf::FixedArray{Float32, Tuple{C}, 1, N}) where {C, N} = Base.OneTo(C)
channel_indices(_buf::Type{FixedArray{Float32, Tuple{C}, 1, N}}) where {C, N} = Base.OneTo(C)
# ...triggers.
sample_indices(_buf::FixedArray{Bool, Tuple{C, S}, 2, N}) where {C, S, N} = Base.OneTo(S)
sample_indices(_buf::Type{FixedArray{Bool, Tuple{C, S}, 2, N}}) where {C, S, N} = Base.OneTo(S)

# View outputs, for sending data going across wires back to the GUI so that it can be displayed
# to the user.
function make_pitch_view_data(pitch)::Vector{Float32}
    assert_is_control_signal_type(typeof(pitch))
    [pitch[1, 1]]
end
function make_audio_view_data(audio)::Vector{Float32}
    assert_is_audio_type(typeof(audio))
    result = Vector{Float32}(undef, 0)
    for sample in a2cs(audio)
        push!(result, sample)
        if length(result) == 64
            break
        end
    end
    result
end
function make_waveform_view_data(waveform::Function)::Vector{Float32}
    assert_is_waveform(waveform)
    result = Vector{Float32}(undef, 64)
    for index in 0:63
        phase = Float32(index) / 63f0
        value = waveform(phase, Int32(1))
        result[index + 1] = value
    end
    result
end
function make_trigger_view_data(trigger)::Vector{Float32}
    assert_is_trigger_type(typeof(trigger))
    result = [0f0]
    for sample in trigger
        if sample
            result[1] = 1f0
            break
        end
    end
    result
end

# Other stuff
lerp(from, to, amount) = to * amount + from * (1 - amount)

# export all
# https://discourse.julialang.org/t/exportall/4970/16
for m in (@__MODULE__, Main.Parameters)
    for n in names(m; all=true)
        if Base.isidentifier(n) && n ∉ (Symbol(@__MODULE__), :eval, :include)
            @eval export $n
        end
    end
end

end # module Lib
