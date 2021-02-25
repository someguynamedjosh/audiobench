function exec()
    # Frequency is a logarithmic scale. Doing log and exp makes it sound correct.
    output = exp.(lerp.(log.(start), log.(end_), sweep))
end