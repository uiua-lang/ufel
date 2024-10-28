## Monadic Functions

| Name | Glyph | Description |
| ---- | ----- | ----------- |
| `identity` | `.` |  Do nothing with an array |
| `negate` | `` ` `` |  Negate an array |
| `not` | `n` |  Not an array |
| `abs` | `b` |  Get the absolute value of an array |
| `sign` | `g` |  Get the sign of an array |
| `length` | `l` |  Get the length of an array |
| `shape` | `h` |  Get the shape of an array |
| `form` | `f` |  Get the form of an array |
| `first` | `a` |  Get the first row of an array |
| `transpose` | `t` |  Rotate the form of an array |

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
| `scan` | `c` |  Scan with a function |

## Dyadic Modifiers

| Name | Glyph | Description |
| ---- | ----- | ----------- |
| `fork` | `^` |  Call two functions on the same sets of values |
| `bracket` | `%` |  Call two functions on different sets of values |

