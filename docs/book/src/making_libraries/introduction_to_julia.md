# Introduction To Julia

This is a quick introduction to Julia for anyone who is familiar with another 
programming language like C++ or Python. If you are just starting out, it is
highly recommended to download Julia from its home page. The download includes
a REPL which allows you to type in Julia code and immediately see its output,
like how Python works.

```julia
# DATA TYPES
@assert typeof(1) == Int64
@assert typeof(Int32(1)) == Int32
@assert typeof(Int32(1) + 1) == Int64
@assert typeof(0.0) == Float64
@assert typeof(0f0) == Float32
@assert 1f3 == 1e3
@assert typeof(1f3) == Float32
@assert typeof(1e3) == Float64

# EXPRESSIONS
123 + 456 * 789
500 == -5_00 * -1
sin(1f0)
exp(log(01f0))
@assert 3^2 == 9

# VARIABLES
variable = 123
use_snake_case = "always"

# ARRAYS
# Regular Julia dynamically sized array
vanilla = [1, 2, 3]
# Because of its heratige of being used for mathematical processing, Julia array
# indices unfortunately start at 1.
@assert vanilla[1] == 1
push!(vanilla, 15)
@assert vanilla[4] == 15

# FUNCTIONS
function add_one(input)
    return input + 1
end
# You can also end a function with an expression 
# and it will be automatically returned.
function add_one(input)
    input + 1
end
# You can also require inputs and return values to be a specific type.
function add_one(input::Int32)::Int32
    # The default integer type is Int64, doing this converts it to an Int32.
    input + Int32(1)
end
# This function will work on any integer type
function add_one(input::Integer)::Integer
    # Int8 is automatically casted up to the appropriate size.
    input + Int8(1)
end

# CONTROL FLOW
if condition1
    println("Condition1 is true")
elseif condition2
    println("Condition2 is true")
else
    println("All conditions are false")
end
total = 0
for step in 1:3
    total += step
end
@assert total == 1 + 2 + 3
while condition
    println("The condition is still true")
end
# If statements can also be expressions if each of their clauses ends with
# an expression.
choice = if prefers_big_numbers 100_000 else 2 end

# STRUCTS
struct NumberHolder
    value::Int32
end
number_holder = NumberHolder(123)
@assert number_holder.value == 123
mutable struct MutableNumberHolder
    value::Int32
end
mutable_number_holder = MutableNumberHolder(123)
@assert mutable_number_holder.value == 123
# Replacing mutable_number_holder with number_holder would cause a compliation error. 
# This is the difference between a mutable struct and a regular struct.
mutable_number_holder.value = 456

# VECTORIZED OPERATORS
# Prefixing an operator with a . means it is 'vectorized' such that it will
# operate on individual elements of each operand. This also serves as a 
# compilation hint to use vectorized instructions like those in SIMD and AVX.
@assert [10, 10, 10] .+ [11, 12, 13] == [21, 22, 23]
# This also works for functions, so that functions do not have to worry about
# implementing the details of iterating over different array types.
@assert abs.([1, -2, -3]) == [1, 2, 3]
# Vectorized operators can also automatically increase the size of a piece of
# data so that it maches another operand.
@assert 10 .+ [11, 12, 13] == [21, 22, 23]
# Vectorized assignment will write data directly to a variable instead of
# collecting it in an intermediate value and then assigning that value to the
# variable.
container = data1 .+ data2 # Intermediate value is created.
container = similar(container) # Create a mutable version of whatever data type
                               # container is.
container .= data1 .+ data2 # Results of individual sums are written directly to 
                            # individual elements of container
# The @. macro will replace all operators on a line or in an expression with
# their vectorized versions.
value = @. abs([1, -2, -3]) + 10
@assert value == [11, 12, 13]
value = similar(value)
@. value = 42
@assert value == [42, 42, 42]
# Frequent use of these features allow Julia to more thorougly optimize the code,
# yielding better performance.
```