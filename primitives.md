## Monadic Functions

| Name | Glyph | Description |
| ---- | ----- | ----------- |
| `identity` | `.` |  Do nothing with an array |
| `negate` | `` ` `` |  Negate an array |
| `not` | `!` |  Not an array |
| `abs` | `b` |  Get the absolute value of an array |
| `sign` | `p` |  Get the sign of an array |
| `floor` | `l` |  Take the floor of an array |
| `ceiling` | `g` |  Take the ceiling of an array |
| `round` | `d` |  Round an array |
| `length` | `n` |  Get the length of an array |
| `shape` | `h` |  Get the shape of an array |
| `form` | `m` |  Get the form of an array |
| `range` | `i` |  Generate a range of numbers or indices for a form |
| `first` | `a` |  Get the first row of an array |
| `reverse` | `z` |  Reverse an array |
| `transpose` | `t` |  Rotate the form of an array |
| `swap` | `w` |  Swap the axes of an array's form |

## Dyadic Functions

| Name | Glyph | Description |
| ---- | ----- | ----------- |
| `add` | `+` |  Add two arrays |
| `subtract` | `-` |  Subtract two arrays |
| `multiply` | `*` |  Multiply two arrays |
| `divide` | `/` |  Divide two arrays |
| `modulo` | `M` |  Modulo two arrays |
| `equal` | `E` |  Check for equality between two arrays |
| `less than` | `L` |  Check if an array is less than another |
| `greater than` | `G` |  Check if an array is greater than another |
| `min` | `N` |  Get the minimum of two arrays |
| `max` | `X` |  Get the maximum of two arrays |
| `chunk` | `C` |  Chunk an array |

## Monadic Modifiers

| Name | Glyph | Description |
| ---- | ----- | ----------- |
| `turn` | `~` |  Call a function considering an array's form vertically rather than horizontally |
| `self` | `'` |  Call a function with two copies of the same value |
| `flip` | `"` |  Call a function with its arguments reversed |
| `dip` | `,` |  Temporarily pop a value from the stack |
| `on` | `o` |  Keep the first argument of a function on top of the stack |
| `by` | `q` |  Keep the last argument of a function below its outputs on the stack |
| `both` | `&` |  Call the same function on two sets of arguments |
| `reduce` | `r` |  Reduce with a function |
| `scan` | `k` |  Scan with a function |
| `fold` | `v` |  Fold a fuction over an array and some accumulators |

## Dyadic Modifiers

| Name | Glyph | Description |
| ---- | ----- | ----------- |
| `fork` | `^` |  Call two functions on the same sets of values |
| `bracket` | `%` |  Call two functions on different sets of values |

