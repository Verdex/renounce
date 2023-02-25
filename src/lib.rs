
#[derive(Debug)]
pub enum ParseError {
    Error,
    Fatal(Vec<Reason>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reason {
    Alt,
    Where,
    End,
    Fatal,
    Rule(&'static str),
}

impl std::fmt::Display for Reason {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        use Reason::*;
        match self {
            Alt => write!(f, "Alternative"),
            Where => write!(f, "Where"),
            End => write!(f, "End"),
            Fatal => write!(f, "Fatal"),
            Rule(r) => write!(f, "Rule: {}", r),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        use ParseError::*;
        match self {
            Error => write!(f, "Error"),
            Fatal(reasons) => write!(f, "Fatal: {}", reasons.iter().map(|r| format!("{}", r)).collect::<Vec<_>>().join("\n")),
        }
    }
}

impl std::error::Error for ParseError {}

#[macro_export]
macro_rules! pat {
    ($name:ident : $in:ty => $out:ty = $pattern : pat => $e:expr) => {
        fn $name(input : &mut impl Iterator<Item = $in>) -> Result<$out, ParseError> {
            match input.next() {
                Some($pattern) => Ok($e),
                _ => Err(ParseError::Error),
            }
        }
    };
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
                    Err(ParseError::Fatal(mut reasons)) => { 
                        reasons.push(Reason::Alt);
                        break 'alt Err(ParseError::Fatal(reasons));
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
            let mut _rp = input.clone();
            parser!(input, _rp, $($rest)*)
        }
    };

    ($input:ident, $rp:ident, ! where $e:expr; $($rest:tt)*) => {
        if $e {
            parser!($input, $rp, $($rest)*)
        }
        else {
            Err(ParseError::Fatal(vec![Reason::Where]))
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

    ($input:ident, $rp:ident, let $name:ident : $t:ty = $e:expr; $($rest:tt)*) => {
        {
            let $name : $t = $e;
            parser!($input, $rp, $($rest)*)
        }
    };

    ($input:ident, $rp:ident, let $name:pat = $e:expr; $($rest:tt)*) => {
        {
            let $name = $e;
            parser!($input, $rp, $($rest)*)
        }
    };

    ($input:ident, $rp:ident, $a:ident <= ! $ma:expr; $($rest:tt)*) => {
        {
            let mut rp = $input.clone();
            match $ma($input) {
                Ok($a) => { 
                    parser!($input, $rp, $($rest)*) 
                },
                Err(ParseError::Fatal(mut reasons)) => { 
                    reasons.push(Reason::Rule(stringify!($a)));
                    Err(ParseError::Fatal(reasons)) 
                },
                Err(ParseError::Error) => { 
                    std::mem::swap($input, &mut rp);
                    Err(ParseError::Fatal(vec![Reason::Rule(stringify!($a))])) 
                }, 
            }
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
                    Err(ParseError::Fatal(mut reasons)) => {
                        reasons.push(Reason::Rule(stringify!($a)));
                        break 'zero_or_more Err(ParseError::Fatal(reasons));
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
                Err(ParseError::Fatal(mut reasons)) => { 
                    reasons.push(Reason::Rule(stringify!($a)));
                    Err(ParseError::Fatal(reasons)) 
                },
            }
        }
    };

    ($input:ident, $rp:ident, $a:ident <= $ma:expr; $($rest:tt)*) => {
        match $ma($input) {
            Ok($a) => {
                parser!($input, $rp, $($rest)*)
            },
            Err(ParseError::Fatal(mut reasons)) => { 
                reasons.push(Reason::Rule(stringify!($a)));
                Err(ParseError::Fatal(reasons)) 
            },
            Err(ParseError::Error) => { 
                std::mem::swap($input, &mut $rp); 
                Err(ParseError::Error) 
            }, 
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
        {
            let mut rp = $input.clone();
            match $input.next() {
                Some(_) => { 
                    std::mem::swap($input, &mut rp);
                    Err(ParseError::Fatal(vec![Reason::End]))
                },
                None => {
                    parser!($input, $rp, $($rest)*)
                },
            }
        }
    };

    ($input:ident, $rp:ident, select $e:expr) => {
        Ok($e)
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::Chars;

    fn return_fatal(_input : &mut impl Iterator<Item = char>) -> Result<char, ParseError> {
        Err(ParseError::Fatal(vec![Reason::Fatal]))
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
            select (one, two)
        })
    }

    #[test]
    fn pat_should_create_parser() {
        let input = [Some(4)];
        let mut input = input.into_iter();

        pat!(p : Option<u8> => u8 = Some(x) => x + 1);

        let output = p(&mut input).expect("the parse should be successful");

        assert_eq!(output, 5);
    }

    #[test]
    fn fatal_end_should_succeed_when_at_end_of_input() {
        let input = "y";
        let mut input = input.chars();

        let output = parser!(input => {
            y <= parse_y;
            ! end;
            select y
        }).expect("the parse should be successful");

        assert_eq!(output, 'y')
    }

    #[test]
    fn fatal_end_should_fail_when_not_at_end_of_input() {
        let input = "ye";
        let mut input = input.chars();

        let output = parser!(input => {
            y <= parse_y;
            ! end;
            select y
        });

        assert!(matches!(output, Err(ParseError::Fatal(_))));
        assert_eq!(input.next(), Some('e'));
    }
    
    #[test]
    fn end_should_succeed_when_at_end_of_input() {
        let input = "y";
        let mut input = input.chars();

        let output = parser!(input => {
            y <= parse_y;
            end;
            select y
        }).expect("the parse should be successful");

        assert_eq!(output, 'y')
    }

    #[test]
    fn end_should_fail_when_not_at_end_of_input() {
        let input = "yy";
        let mut input = input.chars();

        let output = parser!(input => {
            y <= parse_y;
            end;
            select y
        });

        assert!(matches!(output, Err(ParseError::Error)));
        assert_eq!(input.next(), Some('y'));
        assert_eq!(input.next(), Some('y'));
    }

    #[test]
    fn success_fatal_where_should_work() {
        let input = "y";
        let mut input = input.chars();

        let output = parser!(input => {
            y <= parse_y;
            ! where y == 'y';
            select y
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
            select y
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
            select y
        });

        assert!( matches!(output, Err(ParseError::Fatal(_))) );
        assert_eq!( input.next(), None );
    }

    #[test]
    fn where_should_work() {
        let input = "y";
        let mut input = input.chars();

        let output = parser!(input => {
            y <= parse_y;
            where y == 'x';
            select y
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
            select (x, ys)
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
            select ys
        });

        assert!( matches!(output, Err(ParseError::Fatal(_))) );
    }
    
    #[test]
    fn maybe_should_parse_present_item() {
        let input = "yz";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= ? parse_y;
            two <= parse_z;
            select (one, two)
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
            select (zero, one, two)
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
            select (zero, one, two)
        });

        assert!( matches!(output, Err(ParseError::Fatal(_))) );
    }

    #[test]
    fn zero_or_more_should_parse() {
        let input = "yyz";
        let mut input = input.chars();

        let output = parser!(input => {
            ys <= * parse_y;
            _z <= parse_z;
            select ys
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
            select (one, two, three)
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
            select (one, two, three)
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
            select (one, two, three)
        });

        assert!( matches!( output, Err(ParseError::Error) ) );
        assert_eq!( input.next(), Some('y') );
        assert_eq!( input.next(), Some('y') );
    }

    #[test]
    fn where_failure_should_reset_input() {
        let input = "yyz";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= parse_y;
            two <= parse_y;
            where false;
            select (one, two)
        });

        assert!( matches!( output, Err(ParseError::Error) ) );
        assert_eq!( input.next(), Some('y') );
        assert_eq!( input.next(), Some('y') );
        assert_eq!( input.next(), Some('z') );
    } 

    #[test]
    fn rule_failure_should_reset_input() {
        let input = "yyz";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= parse_y;
            two <= parse_y;
            three <= parse_y;
            select (one, two, three)
        });

        assert!( matches!( output, Err(ParseError::Error) ) );
        assert_eq!( input.next(), Some('y') );
        assert_eq!( input.next(), Some('y') );
        assert_eq!( input.next(), Some('z') );
    }

    #[test]
    fn end_failure_should_reset_input() {
        let input = "yyz";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= parse_y;
            two <= parse_y;
            end;
            select (one, two)
        });

        assert!( matches!( output, Err(ParseError::Error) ) );
        assert_eq!( input.next(), Some('y') );
        assert_eq!( input.next(), Some('y') );
        assert_eq!( input.next(), Some('z') );
    }

    #[test]
    fn alt_failure_should_reset_input() {
        let input = "x";
        let mut input = input.chars();

        let output = alt!(input => parse_y; parse_z);

        assert!( matches!( output, Err(ParseError::Error) ) );
        assert_eq!( input.next(), Some('x') );
    }

    #[test]
    fn zero_or_more_failure_should_reset_input() {
        let input = "x";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= * parse_y;
            select one
        });

        assert!( matches!( output, Ok(_) ) );
        assert_eq!( input.next(), Some('x') );
    }

    #[test]
    fn maybe_failure_should_reset_input() {
        let input = "x";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= ? parse_y;
            select one
        });

        assert!( matches!( output, Ok(_) ) );
        assert_eq!( input.next(), Some('x') );
    }

    #[test]
    fn fatal_where_failure_should_not_reset_input() {
        let input = "yyz";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= parse_y;
            two <= parse_y;
            ! where false;
            select (one, two)
        });

        assert!( matches!( output, Err(ParseError::Fatal(_)) ) );
        assert_eq!( input.next(), Some('z') );
    } 

    #[test]
    fn fatal_rule_failure_should_not_reset_input() {
        let input = "yyz";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= parse_y;
            two <= parse_y;
            three <= ! parse_y;
            select (one, two, three)
        });

        assert!( matches!( output, Err(ParseError::Fatal(_)) ) );
        assert_eq!( input.next(), Some('z') );
    }

    #[test]
    fn fatal_end_failure_should_not_reset_input() {
        let input = "yyz";
        let mut input = input.chars();

        let output = parser!(input => {
            one <= parse_y;
            two <= parse_y;
            ! end;
            select (one, two)
        });

        assert!( matches!( output, Err(ParseError::Fatal(_)) ) );
        assert_eq!( input.next(), Some('z') );
    }

    #[test]
    fn fatal_where_should_trace_correctly() {
        fn fatal_where(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                ! where false;
                select '\0'
            })
        }

        fn rule(input : &mut Chars) -> Result<char, ParseError>  {
            parser!(input => {
                _rx <= fatal_where;
                select 'a'
            })
        }
        
        fn fatal_rule(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                _fx <= ! rule;
                select 'a'
            })
        }

        fn maybe(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                mx <= ? fatal_rule;
                let _y : Option<char> = mx;
                select 'a' 
            })
        }

        fn zero_or_more(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                _zx <= * maybe;
                select 'a'
            })
        }

        fn alternate(input : &mut Chars) -> Result<char, ParseError> {
            alt!(input => parse_y; zero_or_more)
        }

        fn let_statement(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                let x = '\0';
                _alt <= alternate;
                select x
            })
        }

        let input = "_";
        let mut input = input.chars();

        let output = let_statement(&mut input);

        if let Err(ParseError::Fatal(reasons)) = output {
            assert_eq!(input.next(), Some('_'));
            assert_eq!(reasons.len(), 7);
            assert_eq!(reasons[0], Reason::Where);
            assert_eq!(reasons[1], Reason::Rule("_rx"));
            assert_eq!(reasons[2], Reason::Rule("_fx"));
            assert_eq!(reasons[3], Reason::Rule("mx"));
            assert_eq!(reasons[4], Reason::Rule("_zx"));
            assert_eq!(reasons[5], Reason::Alt);
            assert_eq!(reasons[6], Reason::Rule("_alt"));
        }
        else {
            assert!(false);
        }
    }

    #[test]
    fn fatal_end_should_trace_correctly() { 
        fn fatal_end(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                ! end;
                select '\0'
            })
        }

        fn rule(input : &mut Chars) -> Result<char, ParseError>  {
            parser!(input => {
                _rx <= fatal_end;
                select 'a'
            })
        }
        
        fn fatal_rule(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                _fx <= ! rule;
                select 'a'
            })
        }

        fn maybe(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                mx <= ? fatal_rule;
                let _y : Option<char> = mx;
                select 'a' 
            })
        }

        fn zero_or_more(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                _zx <= * maybe;
                select 'a'
            })
        }

        fn alternate(input : &mut Chars) -> Result<char, ParseError> {
            alt!(input => parse_y; zero_or_more)
        }

        fn let_statement(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                let x = '\0';
                _alt <= alternate;
                select x
            })
        }

        let input = "_";
        let mut input = input.chars();

        let output = let_statement(&mut input);

        if let Err(ParseError::Fatal(reasons)) = output {
            assert_eq!(input.next(), Some('_'));
            assert_eq!(reasons.len(), 7);
            assert_eq!(reasons[0], Reason::End);
            assert_eq!(reasons[1], Reason::Rule("_rx"));
            assert_eq!(reasons[2], Reason::Rule("_fx"));
            assert_eq!(reasons[3], Reason::Rule("mx"));
            assert_eq!(reasons[4], Reason::Rule("_zx"));
            assert_eq!(reasons[5], Reason::Alt);
            assert_eq!(reasons[6], Reason::Rule("_alt"));
        }
        else {
            assert!(false);
        }
    }

    #[test]
    fn fatal_rule_should_trace_correctly() { 
        fn fatal_rule_origin(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                _y <= ! parse_y;
                select '\0'
            })
        }

        fn rule(input : &mut Chars) -> Result<char, ParseError>  {
            parser!(input => {
                _rx <= fatal_rule_origin;
                select 'a'
            })
        }
        
        fn fatal_rule(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                _fx <= ! rule;
                select 'a'
            })
        }

        fn maybe(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                mx <= ? fatal_rule;
                let _y : Option<char> = mx;
                select 'a' 
            })
        }

        fn zero_or_more(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                _zx <= * maybe;
                select 'a'
            })
        }

        fn alternate(input : &mut Chars) -> Result<char, ParseError> {
            alt!(input => parse_y; zero_or_more)
        }

        fn let_statement(input : &mut Chars) -> Result<char, ParseError> {
            parser!(input => {
                let x = '\0';
                _alt <= alternate;
                select x
            })
        }

        let input = "_";
        let mut input = input.chars();

        let output = let_statement(&mut input);

        if let Err(ParseError::Fatal(reasons)) = output {
            assert_eq!(input.next(), Some('_'));
            assert_eq!(reasons.len(), 7);
            assert_eq!(reasons[0], Reason::Rule("_y"));
            assert_eq!(reasons[1], Reason::Rule("_rx"));
            assert_eq!(reasons[2], Reason::Rule("_fx"));
            assert_eq!(reasons[3], Reason::Rule("mx"));
            assert_eq!(reasons[4], Reason::Rule("_zx"));
            assert_eq!(reasons[5], Reason::Alt);
            assert_eq!(reasons[6], Reason::Rule("_alt"));
        }
        else {
            assert!(false);
        }
    }
}