function exec()
    exponent = octaves + (semitones + cents / 100f0) / 12f0
    transposed = pitch .* (2f0 .^ (exponent .* amount))
end