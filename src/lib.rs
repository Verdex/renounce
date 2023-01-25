
// TODO ParserError definition
// TODO Switch to Result
// TODO zero or more
// TODO rename unit? 
// TODO maybe
// TODO fatal
// TODO error handling 'stack trace'
// TODO probably return failed at item 
// TODO alt definition

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
