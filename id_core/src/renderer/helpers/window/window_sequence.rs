#![allow(non_snake_case)]

use super::{UserContext, WindowSetup};

// Implement From for tuples of size 1 to 15
// Each implementation converts a tuple of WindowSetup<UC> items into a Vec<Box<dyn WindowSetup<UC>>>

pub struct WindowSequence<UC: UserContext + 'static> {
    pub sequence: Vec<Box<dyn WindowSetup<UC>>>,
}

// Macro to implement From for tuples of size 1 to 15
macro_rules! impl_from_tuples {
    ($( $Tuple:ident { $($T:ident),+ } ),+ $(,)?) => {
        $(
            impl<UC: UserContext, $($T),+> From<($($T,)+)> for WindowSequence<UC>
            where
                $($T: WindowSetup<UC> + 'static),+
            {
                fn from(tuple: ($($T,)+)) -> Self {
                    // Unpack the tuple into individual variables
                    let ($($T,)+) = tuple;

                    // Create a vector with the expected type
                    let sequence: Vec<Box<dyn WindowSetup<UC>>> = vec![
                        $(
                            Box::new($T),
                        )+
                    ];

                    WindowSequence { sequence }
                }
            }
        )+
    };
}

// Use the macro to generate implementations for tuples of size 1 to 15
impl_from_tuples!(
    Tuple1 { T1 },
    Tuple2 { T1, T2 },
    Tuple3 { T1, T2, T3 },
    Tuple4 { T1, T2, T3, T4 },
    Tuple5 { T1, T2, T3, T4, T5 }
);
