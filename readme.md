# Ufel

Ufel ([**U**iua](https://uiua.org) **f**orm **e**xperimentation **l**anguage) is a language for experementing with higher-order arrays.

In array languages, an array has a properly called its *shape*, which is a list of natural numbers than describes how elements are layed out along the array's axes.

In array languages like APL, J, BQN, Kap, TinyAPL, and Uiua, the shape of an array is always 1-dimensional.
Ufel explores the possibility of multi-dimensional shapes.

Ufel should not be used for anything other than experimentation. Features are subject to massive change.

See [primitives.md](primitives.md) for a list of primitives.

## Installation

To install the interpreter, you must build from source with Rust.

```sh
git clone https://github.com/uiua-lang/ufel
cargo install --path ufel
```

The Ufel file extension is `.fel`.

## Tutorial

This tutorial assumes that you have at least a basic understanding of [Uiua](https://uiua.org).

Like Uiua, Ufel is a stack-based array language.

Unlike Uiua, Ufel is typed with only ASCII characters. It also runs left-to-right, rather than right-to-left.

Ufel retains Uiua's stack array notation.

```ufel
[1 2 3] 4 +
# [5 6 7]
```

```ufel
[5 8 2] [2 2 1] -
# [3 6 1]
```

`(i)range` generates a range of numbers.
```ufel
5 i
# [0 1 2 3 4]
```

`(n)length` gets the length of an array.
```ufel
[1 2 3] n
# 3
```

```ufel
[[1 2] [3 4]] n
# 2
```

`(r)reduce` reduces an array with a function.
```ufel
[1 2 3 4] r+
# 10
```

`(s)scan` scans an array with a function.

`(h)shape` gets the shape of an array.
```ufel
[1 2 3] h
# [3]
```

```ufel
[[1 2 3] [4 5 6]] h
# [2 3]
```

The main thing that separates Ufel from other array languages is that the way an array's elements are laid out is not fully specified by a 1-dimensional vector shape.

Arrays in Ufel have their axes specified in a *matrix*. This is called the array's *form*.

Form is to shape as shape is to length. A form is always 2-dimensional.

You can get the form of an array with `(m)form`.

```ufel
[1 2 3] m
# ╭─
# ╷ 3
#     ╯
```

```ufel
[[1 2] [3 4]] m
# ╭─
# ╷ 2 2
#       ╯
```

So far, these forms just look like normal shapes with an extra dimension. We call arrays like this *normal*.

Form axes are specified by their *orientation*, either horizontal or vertical. A normal array has a vertical rank of 1.

If we use the `(~)turn` modifier, we can arrange an array vertically rather than horizontally.

```ufel
~[[1 2] [3 4]] m
# ╭─
# ╷ 2
#   2
#     ╯
```

And now a bizarre thing happens if we ask for the `(n)length` or `(h)shape`!

```ufel
~[[1 2] [3 4]] n
# 4
```

```ufel
~[[1 2] [3 4]] h
# [2]
```

This is because the array is entirely vertical, but functions operate horizontally by default.

We can tell a function to operate vertically with `(~)turn`.


```ufel
~[[1 2] [3 4]] ~n
# 2
```

```ufel
~[[1 2] [3 4]] ~h
# [2 2]
```

Alternatively, we can use `(w)swap` to swap the vertical and horizontal form axes.

```ufel
~[[1 2] [3 4]] w n
# 2
```

The `(C)chunk` function expands the axes of an array into the vertical part of its form. It takes an array and a size.

```ufel
10i 5C
# ╭─
# ╷ 0 1 2 3 4
#   5 6 7 8 9
#             ╯
```

This can be used along with the `(v)fold` modifier to approximate a reshape.

The `(')self` modifier duplicates the stop stack value before calling its function.

```ufel
16i [4 2] vCw 'h
# ╭─
# ╷  0  1  2  3
# ╷  4  5  6  7
# 
#    8  9 10 11
#   12 13 14 15
#               ╯
# [2 2 4]
```

Notice that if we leave all the axes vertical instead of `(w)swap`ping, modifiers like `(r)reduce` will ignore those axes by default.

```ufel
16i [4 2] vC r+
# 120
```

```ufel
16i [4 2] vC ~r+
# ╭─
# ╷  1  5  9 13
#   17 21 25 29
#               ╯
```