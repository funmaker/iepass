# Notes

# Pico8
- decimal numeric literals are parsed into f64 and then truncates into Q16.16, wrapping on overflows
- hex/binary numeric literals parsed directly into Q16.16, truncated, wrapping on overflows
- tostr displays up to 4 decimal places, truncated
- arithmetic wraps on overflow
- modulo is euclidean (corresponds to \ operator)
- left shift is logical
