
// TODO ParserError definition
// TODO rename unit? (to result?) 
// TODO error handling 'stack trace'
// TODO probably return failed at item 
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

    ($input:ident, $rp:ident, ! where $e:expr; $($rest:tt)*) => {
        if $e {
            parser!($input, $rp, $($rest)*)
        }
        else {
            std::mem::swap($input, &mut $rp);
            Err(ParseError::Fatal)
        }
    };

    ($input:ident, $rp:ident, where $e:expr; $($rest:tt)*) => {
        if $e {
            parser!($input, $rp, $($rest)*)
        }
        else {
            std::mem::swap($input, &mut $rp);
            Err(ParseError::Error)
        }
    };

    ($input:ident, $rp:ident, let $name:pat = $e:expr; $($rest:tt)*) => {
        {
            let $name = $e;
            parser!($input, $rp, $($rest)*)
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
        {
            let mut rp = $input.clone();

            match $ma($input) {
                Ok(x) => {
                    let $a = Some(x);
                    parser!($input, $rp, $($rest)*)
                },
                Err(ParseError::Error) => { 
                    std::mem::swap($input, &mut rp); 
                    let $a = None;
                    parser!($input, $rp, $($rest)*)
                }, 
                Err(ParseError::Fatal) => { std::mem::swap($input, &mut $rp); Err(ParseError::Fatal) },
            }
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

    ($input:ident, $rp:ident, end; $($rest:tt)*) => {
        match $input.next() {
            Some(_) => { std::mem::swap($input, &mut $rp); Err(ParseError::Error) },
            None => {
                parser!($input, $rp, $($rest)*)
            },
        }
    };

    ($input:ident, $rp:ident, ! end; $($rest:tt)*) => {
        match $input.next() {
            Some(_) => { std::mem::swap($input, &mut $rp); Err(ParseError::Fatal) },
            None => {
                parser!($input, $rp, $($rest)*)
            },
        }
    };

    ($input:ident, $rp:ident, unit $e:expr) => {
        Ok($e)
    };
}

#[cfg(test)]
mod test {
    use super::*;

    fn return_fatal(_input : &mut impl Iterator<Item = char>) -> Result<char, ParseError> {
        Err(ParseError::Fatal)
    }

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
    fn success_fatal_where_should_work() {
        let input = "y";
        let mut input = input.chars();

        let output = parser!(input => {
            y <= parse_y;
            ! where y == 'y';
            unit y
        }).expect("the parse should be successful");

        assert_eq!(output, 'y');
    }

    #[test]
    fn success_where_should_work() {
        let input = "y";
        let mut input = input.chars();

        let output = parser!(input => {
            y <= parse_y;
            where y == 'y';
            unit y
        }).expect("the parse should be successful");

        assert_eq!(output, 'y');
    }

    #[test]
    fn fatal_where_should_work() {
        let input = "y";
        let mut input = input.chars();

        let output = parser!(input => {
            y <= parse_y;
            ! where y == 'x';
            unit y
        });

        assert!( matches!(output, Err(ParseError::Fatal)) );
        assert_eq!( input.next(), Some('y') );
    }

    #[test]
    fn where_should_work() {
        let input = "y";
        let mut input = input.chars();

        let output = parser!(input => {
            y <= parse_y;
            where y == 'x';
            unit y
        });

        assert!( matches!(output, Err(ParseError::Error)) );
        assert_eq!( input.next(), Some('y') );
    }

    #[test]
    fn let_should_work() {
        struct X(u8);

        let input = "yyyyyy";
        let mut input = input.chars();

        let output = parser!(input => {
            let X(x) = X(1);
            ys <= * parse_yy;
            unit (x, ys)
        }).expect("the parse should be successful");

        assert_eq!(output.0, 1);
        assert_eq!(output.1, [('y', 'y'), ('y', 'y'), ('y', 'y')]);
    }

    #[test]
    fn zero_or_more_should_pass_through_fatal() {
        let input = "yyyyyz";
        let mut input = input.chars();

        let output = parser!(input => {
            ys <= * parse_yy;
            unit ys
        });

        assert!( matches!(output, Err(ParseError::Fatal)) );
    }
    
    #[test]
    fn maybe_should_parse_present_item() {
        let input = "yz";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= ? parse_y;
            two <= parse_z;
            unit (one, two)
        }).expect("the parse should be successful");

        assert_eq!(output, (Some('y'), 'z'));
    }

    #[test]
    fn maybe_should_reset_input_to_immediately_before_it_on_failure() {
        let input = "wz";
        let mut input = input.chars();

        let output = parser!(input => {
            zero <= any_char;
            one <= ? parse_y;
            two <= parse_z;
            unit (zero, one, two)
        }).expect("the parse should be successful");

        assert_eq!(output, ('w', None, 'z'));
    }

    #[test]
    fn maybe_should_pass_through_fatal() {
        let input = "wz";
        let mut input = input.chars();

        let output = parser!(input => {
            zero <= any_char;
            one <= ? return_fatal;
            two <= parse_z;
            unit (zero, one, two)
        });

        assert!( matches!(output, Err(ParseError::Fatal)) );
    }

    #[test]
    fn zero_or_more_should_parse() {
        let input = "yyz";
        let mut input = input.chars();

        let output = parser!(input => {
            ys <= * parse_y;
            _z <= parse_z;
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