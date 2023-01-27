
// TODO ParserError definition
// TODO zero or more
// TODO rename unit? 
// TODO error handling 'stack trace'
// TODO probably return failed at item 
// TODO alt definition
// TODO where (and where fatal) 
// TODO let

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
            let mut rp = input.clone();
            parser!(input, rp, $($rest)*)
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

    ($input:ident, $rp:ident, $a:ident <= ! $ma:expr; $($rest:tt)*) => {
        match $ma($input) {
            Ok($a) => { 
                parser!($input, $rp, $($rest)*) 
            },
            Err(ParseError::Fatal) => { std::mem::swap($input, &mut $rp); Err(ParseError::Fatal) },
            Err(ParseError::Error) => { std::mem::swap($input, &mut $rp); Err(ParseError::Fatal) }, 
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

    fn parse_y(input : &mut impl Iterator<Item = char>) -> Result<char, ParseError> {
        match input.next() {
            Some('y') => Ok('y'),
            _ => Err(ParseError::Error),
        }
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
        let input = "yxz";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= parse_y;
            two <= parse_y;
            three <= parse_y;
            unit (one, two, three)
        });

        assert!( matches!( output, Err(ParseError::Error) ) );
        assert_eq!( input.next(), Some('y') );
    }
}