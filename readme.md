# Ufel

Ufel ([**U**iua](https://uiua.org) **f**orm **e**xperimentation **l**anguage) is a language for experementing with higher-order arrays.

In array languages, an array has a properly called its *shape*, which is a list of natural numbers than describes how elements are layed out along the array's axes.

In array languages like APL, J, BQN, Kap, TinyAPL, and Uiua, the shape of an array is always 1-dimensional.
Ufel explores the possibility of multi-dimensional shapes.

The Ufel file extension is `.fel`.

## How it Works

Like Uiua, Ufel is a stack-based array language.

Unlike Uiua, Ufel is typed with only ASCII characters. It also runs left-to-right, rather than right-to-left.

Ufel retain Uiua's stack array notation

```ufel
[1 2 3] 4 +
# [5 6 7]
```

```ufel
[5 8 2] [2 2 1] -
# [3 6 1]
```

See [primitives.md](primitives.md) for a list of primitives.