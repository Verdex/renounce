#[macro_export]
macro_rules! parser {
    ($input:ident => { $($rest:tt)* } )  => {
        {
            let input = &mut $input;
            let mut rp = input.clone();
            parser!(input, rp, $($rest)*)
        }
    };

    ($input:ident, $rp:ident, $a:ident <= $ma:expr; $($rest:tt)*) => {
        match $ma($input) {
            Some($a) => {
                parser!($input, $rp, $($rest)*)
            },
            None => { std::mem::swap($input, &mut $rp); None },
        }
    };

    ($input:ident, $rp:ident, unit $e:expr) => {
        Some($e)
    };
}
