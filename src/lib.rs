
// TODO ParserError definition
// TODO rename unit? 
// TODO error handling 'stack trace'
// TODO probably return failed at item 
// TODO where (and where fatal) 
// TODO let
// TODO end of stream

#[derive(Debug)]
pub enum ParseError {
    Error,
    Fatal,
}

#[macro_export]
macro_rules! alt {
    ($input:ident => $($parser:expr);* ) => {
        'alt : {
            use std::borrow::BorrowMut;
            let input = $input.borrow_mut();

            $(
                let mut rp = input.clone();
                match $parser(input) {
                    Ok(x) => { break 'alt Ok(x); },
                    Err(ParseError::Error) => { std::mem::swap(input, &mut rp); },
                    Err(ParseError::Fatal) => { 
                        std::mem::swap(input, &mut rp);  
                        break 'alt Err(ParseError::Fatal);
                    },
                }
            )*

            Err(ParseError::Error)
        }
    };
}

#[macro_export]
macro_rules! parser {
    ($input:ident => { $($rest:tt)* } )  => {
        {
            use std::borrow::BorrowMut;
            let input = $input.borrow_mut();
            let mut rp = input.clone();
            parser!(input, rp, $($rest)*)
        }
    };

    ($input:ident, $rp:ident, $a:ident <= ! $ma:expr; $($rest:tt)*) => {
        match $ma($input) {
            Ok($a) => { 
                parser!($input, $rp, $($rest)*) 
            },
            Err(ParseError::Fatal) => { std::mem::swap($input, &mut $rp); Err(ParseError::Fatal) },
            Err(ParseError::Error) => { std::mem::swap($input, &mut $rp); Err(ParseError::Fatal) }, 
        }
    };

    ($input:ident, $rp:ident, $a:ident <= * $ma:expr; $($rest:tt)*) => {
        'zero_or_more : {
            let mut ret = vec![];
            loop {
                let mut peek = $input.clone();
                match $ma($input) {
                    Ok(x) => { ret.push(x); },
                    Err(ParseError::Error) => {
                        std::mem::swap($input, &mut peek); 
                        break;
                    },
                    Err(ParseError::Fatal) => {
                        std::mem::swap($input, &mut $rp); 
                        break 'zero_or_more Err(ParseError::Fatal);
                    },
                }
            }
            let $a = ret;
            parser!($input, $rp, $($rest)*)
        }
    };

    ($input:ident, $rp:ident, $a:ident <= ? $ma:expr; $($rest:tt)*) => {
        match $ma($input) {
            Ok($a) => {
                parser!($input, $($rest)*)
            },
            Err(ParseError::Error) => { 
                std::mem::swap($input, &mut $rp); 
                let $a = None;
                parser!($input, $rp, $($rest)*)
            }, 
            Err(ParseError::Fatal) => { std::mem::swap($input, &mut $rp); Err(ParseError::Fatal) },
        }
    };

    ($input:ident, $rp:ident, $a:ident <= $ma:expr; $($rest:tt)*) => {
        match $ma($input) {
            Ok($a) => {
                parser!($input, $rp, $($rest)*)
            },
            Err(ParseError::Fatal) => { std::mem::swap($input, &mut $rp); Err(ParseError::Fatal) },
            Err(ParseError::Error) => { std::mem::swap($input, &mut $rp); Err(ParseError::Error) }, 
        }
    };

    ($input:ident, $rp:ident, unit $e:expr) => {
        Ok($e)
    };
}

#[cfg(test)]
mod test {
    use super::*;

    fn any_char(input : &mut impl Iterator<Item = char>) -> Result<char, ParseError> {
        match input.next() {
            Some(x) => Ok(x),
            None => Err(ParseError::Error),
        }
    }

    fn parse_z(input : &mut impl Iterator<Item = char>) -> Result<char, ParseError> {
        match input.next() {
            Some('z') => Ok('z'),
            _ => Err(ParseError::Error),
        }
    }

    fn parse_y(input : &mut impl Iterator<Item = char>) -> Result<char, ParseError> {
        match input.next() {
            Some('y') => Ok('y'),
            _ => Err(ParseError::Error),
        }
    }

    fn parse_yy(input : &mut (impl Iterator<Item = char> + Clone)) -> Result<(char, char), ParseError> {
        parser!(input => {
            one <= parse_y;
            two <= ! parse_y;
            unit (one, two)
        })
    }

    #[test]
    fn zero_or_more_should_parse() {
        let input = "yyz";
        let mut input = input.chars();

        let output = parser!(input => {
            ys <= * parse_y;
            unit ys
        }).expect("the parse should be successful");

        assert_eq!(output, ['y', 'y']);
    }

    #[test]
    fn alt_should_be_usable_in_parse() {
        fn y_or_z(input : &mut (impl Iterator<Item = char> + Clone)) -> Result<char, ParseError> {
            alt!(input => parse_y; parse_z)
        }

        let input = "yzy";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= y_or_z; 
            two <= y_or_z; 
            three <= y_or_z; 
            unit (one, two, three)
        }).expect("the parse should be successful");

        assert_eq!(output, ('y', 'z', 'y'));
    }

    #[test]
    fn alt_should_parse_successfully() {
        let input = "x";
        let mut input = input.chars();

        let output = alt!(input => parse_y; parse_z; any_char).expect("the parse should be successful");

        assert_eq!(output, 'x');
    }

    #[test]
    fn simple_parser_should_parse() {
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

    #[test]
    fn failed_parse_should_return_input_to_original_state() {
        let input = "yyz";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= parse_y;
            two <= parse_y;
            three <= parse_y;
            unit (one, two, three)
        });

        assert!( matches!( output, Err(ParseError::Error) ) );
        assert_eq!( input.next(), Some('y') );
        assert_eq!( input.next(), Some('y') );
    }
}