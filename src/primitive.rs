use std::fmt;

use enum_iterator::{all, All, Sequence};

primitive!(Mon(Monadic), Dy(Dyadic), MonMod(Mod), DyMod(DyMod));

prim!(Monadic,
    /// Do nothing with an array
    (Identity, "identity", '.'),
    /// Negate an array
    (Neg, "negate", '`'),
    /// Get the length of an array
    (Len, "length", 'l'),
    /// Get the shape of an array
    (Shape, "shape", 'h'),
    /// Get the form of an array
    (Form, "form", 'f'),
);
prim!(Dyadic,
    /// Add two arrays
    (Add, "add", '+'),
    /// Subtract two arrays
    (Sub, "subtract", '-'),
    /// Multiply two arrays
    (Mul, "multiply", '*'),
    /// Divide two arrays
    (Div, "divide", '/'),
    /// Modulo two arrays
    (Mod, "modulo", 'M'),
    /// Check for equality between two arrays
    (Eq, "equal", 'E'),
    /// Check for inequality between two arrays
    (Neq, "not equal", 'N'),
    /// Check if an array is less than another
    (Lt, "less than", 'L'),
    /// Check if an array is greater than another
    (Gt, "greater than", 'G'),
);
prim!(Mod,
    /// Temporarily pop a value from the stack
    (Dip, "dip", ','),
    /// Reduce with a function
    (Reduce, "reduce", 'r'),
    /// Scan with a function
    (Scan, "scan", 's'),
);
prim!(DyMod,
    /// Call two functions on the same sets of values
    (Fork, "fork", 'F'),
);

pub trait PrimKind: Sized + Sequence {
    fn glyph(&self) -> char;
    fn name(&self) -> &'static str;
    fn from_glyph(c: char) -> Option<Self>;
    fn from_name(name: &str) -> Option<Self>;
    fn description(&self) -> &'static str;
    fn full_docs(&self) -> &'static str;
    fn all() -> All<Self> {
        all::<Self>()
    }
}

#[cfg(test)]
#[test]
fn gen_prim_tables() {
    type Props<'a> = Vec<(&'a str, char, &'a str)>;
    let kinds: Vec<(&str, Props)> = vec![
        (
            "Monadic Functions",
            Monadic::all()
                .map(|p| (p.name(), p.glyph(), p.description()))
                .collect(),
        ),
        (
            "Dyadic Functions",
            Dyadic::all()
                .map(|p| (p.name(), p.glyph(), p.description()))
                .collect(),
        ),
        (
            "Monadic Modifiers",
            Mod::all()
                .map(|p| (p.name(), p.glyph(), p.description()))
                .collect(),
        ),
        (
            "Dyadic Modifiers",
            DyMod::all()
                .map(|p| (p.name(), p.glyph(), p.description()))
                .collect(),
        ),
    ];

    let mut md = String::new();
    for (name, items) in kinds {
        md.push_str("## ");
        md.push_str(name);
        md.push_str("\n\n");
        md.push_str("| Name | Glyph | Description |\n");
        md.push_str("| ---- | ----- | ----------- |\n");
        for (name, glyph, desc) in items {
            md.push_str("| `");
            md.push_str(name);
            md.push_str("` | `");
            md.push_str(&if glyph == '`' {
                "` ` `".into()
            } else {
                glyph.to_string()
            });
            md.push_str("` | ");
            md.push_str(desc);
            md.push_str(" |\n");
        }
        md.push('\n');
    }

    std::fs::write("primitives.md", md).unwrap();
}

macro_rules! primitive {
    ($($name:ident($ty:ty)),* $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Sequence)]
        pub enum Primitive {
            $($name($ty),)*
        }

        impl fmt::Debug for Primitive {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Self::$name(p) => p.fmt(f),)*
                }
            }
        }

        impl fmt::Display for Primitive {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Self::$name(p) => p.fmt(f),)*
                }
            }
        }

        impl PrimKind for Primitive {
            fn glyph(&self) -> char {
                match self {
                    $(Self::$name(p) => p.glyph(),)*
                }
            }
            fn name(&self) -> &'static str {
                match self {
                    $(Self::$name(p) => p.name(),)*
                }
            }
            fn from_glyph(c: char) -> Option<Self> {
                None $(.or_else(|| <$ty>::from_glyph(c).map(Self::$name)))*
            }
            fn from_name(name: &str) -> Option<Self> {
                None $(.or_else(|| <$ty>::from_name(name).map(Self::$name)))*
            }
            fn description(&self) -> &'static str {
                match self {
                    $(Self::$name(p) => p.description(),)*
                }
            }
            fn full_docs(&self) -> &'static str {
                match self {
                    $(Self::$name(p) => p.full_docs(),)*
                }
            }
        }

        $(
            impl From<$ty> for Primitive {
                fn from(p: $ty) -> Self {
                    Self::$name(p)
                }
            }
        )*
    }
}
use primitive;

macro_rules! prim {
    (   $prim:ident,
        $(
            #[doc = $doc:literal]
            $(#[doc = $doc2:literal])*
            ($variant:ident, $name:literal, $glyph:literal)
        ),* $(,)?
    ) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Sequence)]
        pub enum $prim {
            $($variant,)*
        }

        impl fmt::Debug for $prim {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Self::$variant => write!(f, "{} {}", $glyph, $name),)*
                }
            }
        }

        impl fmt::Display for $prim {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Self::$variant => write!(f, "{}", $glyph),)*
                }
            }
        }

        impl PrimKind for $prim {
            fn glyph(&self) -> char {
                match self {
                    $(Self::$variant => $glyph,)*
                }
            }
            fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)*
                }
            }
            fn from_glyph(c: char) -> Option<Self> {
                match c {
                    $($glyph => Some(Self::$variant),)*
                    _ => None,
                }
            }
            fn from_name(name: &str) -> Option<Self> {
                match name {
                    $($name => Some(Self::$variant),)*
                    _ => None,
                }
            }
            fn description(&self) -> &'static str {
                match self {
                    $(Self::$variant => $doc,)*
                }
            }
            fn full_docs(&self) -> &'static str {
                match self {
                    $(Self::$variant => $doc,)*
                }
            }
        }
    }
}
use prim;
