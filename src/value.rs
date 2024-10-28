#![allow(clippy::should_implement_trait)]

use crate::{pervade::*, Array, Ufel, UfelResult};

mon!(neg);
mon!(not);
mon!(abs);
mon!(sign);

dy!(add);
dy!(sub);
dy!(mul);
dy!(div);
dy!(mod_);
dy!(eq);
dy!(ne);
dy!(lt);
dy!(gt);
dy!(min);
dy!(max);

macro_rules! mon {
    ($name:ident) => {
        impl Array {
            pub fn $name(mut self) -> Self {
                for elem in self.data.as_mut_slice() {
                    *elem = $name::num(*elem);
                }
                self
            }
        }
    };
}
use mon;

macro_rules! dy {
    ($name:ident) => {
        impl Array {
            pub fn $name(
                self,
                other: Self,
                a_depth: usize,
                b_depth: usize,
                rt: &Ufel,
            ) -> UfelResult<Self> {
                pervade(self, other, a_depth, b_depth, $name::num_num, rt)
            }
        }
    };
}
use dy;
