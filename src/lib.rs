
// TODO ParserError definition
// TODO zero or more
// TODO rename unit? 
// TODO error handling 'stack trace'
// TODO probably return failed at item 
// TODO alt definition

#[derive(Debug)]
pub enum ParseError {
    Error,
    Fatal,
}

#[macro_export]
macro_rules! parser {
    ($input:ident => { $($rest:tt)* } )  => {
        {
            let input = &mut $input;
            parser!(input, $($rest)*)
        }
    };

    ($input:ident, $a:ident <= $ma:expr; $($rest:tt)*) => {
        {
            let mut rp = $input.clone(); 
            match $ma($input) {
                Ok($a) => {
                    parser!($input, $($rest)*)
                },
                Err(ParseError::Fatal) => { std::mem::swap($input, &mut rp); Err(ParseError::Fatal) },
                Err(ParseError::Error) => { std::mem::swap($input, &mut rp); Err(ParseError::Error) }, 
            }
        }
    };

    ($input:ident, $a:ident <= ! $ma:expr; $($rest:tt)*) => {
        {
            let mut rp = $input.clone(); 
            match $ma($input) {
                Ok($a) => { 
                    parser!($input, $($rest)*) 
                },
                Err(ParseError::Fatal) => { std::mem::swap($input, &mut rp); Err(ParseError::Fatal) },
                Err(ParseError::Error) => { std::mem::swap($input, &mut rp); Err(ParseError::Fatal) }, 
            }
        }
    };

    ($input:ident, $a:ident <= ? $ma:expr; $($rest:tt)*) => {
        {
            let mut rp = $input.clone(); 
            match $ma($input) {
                Ok($a) => {
                    parser!($input, $($rest)*)
                },
                Err(ParseError::Error) => { 
                    std::mem::swap($input, &mut $rp); 
                    let $a = None;
                    parser!($input, $($rest)*)
                }, 
                Err(ParseError::Fatal) => { std::mem::swap($input, &mut rp); Err(ParseError::Fatal) },
            }
        }
    };

    ($input:ident, unit $e:expr) => {
        Ok($e)
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_parser_should_parse() {
        fn parse_y(input : &mut impl Iterator<Item = char>) -> Result<char, ParseError> {
            match input.next() {
                Some('y') => Ok('y'),
                _ => Err(ParseError::Error),
            }
        }

        let input = "yyy";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= parse_y;
            two <= parse_y;
            three <= parse_y;
            unit (one, two, three)
        }).expect("the parse should be successful");

        assert_eq!( output, ('y', 'y', 'y') );
    }
}