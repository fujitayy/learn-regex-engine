use anyhow::{anyhow, Error};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, none_of},
    multi::{many1, separated_list1},
};

fn main() -> Result<(), Error> {
    let re = Regex::parse("(...(-|=)....,* *)*")?;
    let s = "150-0022,150-0023, 150-0024,   150=0025";
    let r = re.test(s);
    println!("{r:?}");

    Ok(())
}

#[derive(Debug)]
enum NFA {
    Head(Head),
    Character(Character),
    Dot(Dot),
    Choice(Choice),
    Repeat(Repeat),
    Group(Group),
    Accept,
}

impl NFA {
    fn head() -> Box<NFA> {
        Box::new(NFA::Head(Head { next: None }))
    }

    fn character(character: char) -> Box<NFA> {
        Box::new(NFA::Character(Character {
            character,
            next: None,
        }))
    }

    fn dot() -> Box<NFA> {
        Box::new(NFA::Dot(Dot { next: None }))
    }

    fn choice(choices: Vec<Box<NFA>>) -> Box<NFA> {
        Box::new(NFA::Choice(Choice {
            choices,
            next: None,
        }))
    }

    fn repeat(nfa: Box<NFA>) -> Box<NFA> {
        Box::new(NFA::Repeat(Repeat { nfa, next: None }))
    }

    fn group(nfa: Box<NFA>) -> Box<NFA> {
        Box::new(NFA::Group(Group { nfa, next: None }))
    }

    fn accept() -> Box<NFA> {
        Box::new(NFA::Accept)
    }
}

#[derive(Debug)]
struct Head {
    next: Option<Box<NFA>>,
}

#[derive(Debug)]
struct Dot {
    next: Option<Box<NFA>>,
}

#[derive(Debug)]
struct Character {
    character: char,
    next: Option<Box<NFA>>,
}

#[derive(Debug)]
struct Choice {
    choices: Vec<Box<NFA>>,
    next: Option<Box<NFA>>,
}

#[derive(Debug)]
struct Repeat {
    nfa: Box<NFA>,
    next: Option<Box<NFA>>,
}

#[derive(Debug)]
struct Group {
    nfa: Box<NFA>,
    next: Option<Box<NFA>>,
}

#[derive(Debug)]
struct Regex {
    nfa: Box<NFA>,
}

impl Regex {
    fn test(&self, input: &str) -> bool {
        let char_vec: Vec<char> = input.chars().collect();
        return self.nfa.test(&char_vec).is_some();
    }

    fn parse(input: &str) -> Result<Self, Error> {
        let (_, nfa) = parse_as_nfa(input).map_err(|e| anyhow!("{e}"))?;
        return Ok(Regex { nfa });
    }
}

trait Tester {
    fn test<'a>(&self, input: &'a [char]) -> Option<&'a [char]>;
}

impl Tester for NFA {
    fn test<'a>(&self, input: &'a [char]) -> Option<&'a [char]> {
        match self {
            NFA::Head(head) => {
                return head.test(input);
            }
            NFA::Character(character) => {
                return character.test(input);
            }
            NFA::Dot(dot) => {
                return dot.test(input);
            }
            NFA::Choice(choice) => {
                return choice.test(input);
            }
            NFA::Repeat(repeat) => {
                return repeat.test(input);
            }
            NFA::Group(group) => {
                return group.test(input);
            }
            NFA::Accept => {
                return Some(input);
            }
        }
    }
}

impl Tester for Head {
    fn test<'a>(&self, input: &'a [char]) -> Option<&'a [char]> {
        let next = self.next.as_ref()?;
        return next.test(input);
    }
}

impl Tester for Dot {
    fn test<'a>(&self, input: &'a [char]) -> Option<&'a [char]> {
        let next = self.next.as_ref()?;
        if input.is_empty() {
            return None;
        } else {
            return next.test(&input[1..]);
        };
    }
}

impl Tester for Character {
    fn test<'a>(&self, input: &'a [char]) -> Option<&'a [char]> {
        let next = self.next.as_ref()?;
        if input.get(0) == Some(&self.character) {
            return next.test(&input[1..]);
        } else {
            return None;
        }
    }
}

impl Tester for Choice {
    fn test<'a>(&self, input: &'a [char]) -> Option<&'a [char]> {
        let next = self.next.as_ref()?;

        let remain = self
            .choices
            .iter()
            .filter_map(|choice| choice.test(input))
            .next()?;

        return next.test(remain);
    }
}

impl Tester for Repeat {
    fn test<'a>(&self, input: &'a [char]) -> Option<&'a [char]> {
        fn go<'a>(input: &'a [char], nfa: &Box<NFA>, next: &Box<NFA>) -> Option<&'a [char]> {
            return nfa
                .test(input)
                .and_then(|remain| go(remain, nfa, next))
                .or_else(|| next.test(input));
        }

        let next = self.next.as_ref()?;
        return go(input, &self.nfa, next);
    }
}

impl Tester for Group {
    fn test<'a>(&self, input: &'a [char]) -> Option<&'a [char]> {
        let next = self.next.as_ref()?;
        let remain = self.nfa.test(input)?;
        return next.test(remain);
    }
}

fn parse_as_nfa(input: &str) -> nom::IResult<&str, Box<NFA>> {
    maybe_choice(input)
}

fn maybe_choice(input: &str) -> nom::IResult<&str, Box<NFA>> {
    let (remain, choices) = separated_list1(tag("|"), many1(maybe_repeat))(input)?;

    if choices.len() == 1 {
        let first = choices.into_iter().next().unwrap();
        let nfa = fold_nfa_states(first);
        return Ok((remain, nfa));
    } else {
        let choices = choices.into_iter().map(fold_nfa_states).collect();
        let nfa = fold_nfa_states(vec![NFA::choice(choices)]);
        return Ok((remain, nfa));
    }
}

fn maybe_repeat(input: &str) -> nom::IResult<&str, Box<NFA>> {
    let (remain, partial) = alt((group, character, dot))(input)?;

    let r: nom::IResult<&str, char> = char('*')(remain);
    match r {
        Ok((remain, _)) => {
            let nfa = fold_nfa_states(vec![partial]);
            return Ok((remain, NFA::repeat(nfa)));
        }
        Err(_) => {
            return Ok((remain, partial));
        }
    }
}

fn group(input: &str) -> nom::IResult<&str, Box<NFA>> {
    let (remain, _) = char('(')(input)?;
    let (remain, nfa) = parse_as_nfa(remain)?;
    let (remain, _) = char(')')(remain)?;
    return Ok((remain, NFA::group(nfa)));
}

fn character(input: &str) -> nom::IResult<&str, Box<NFA>> {
    let (remain, c) = none_of(".*()|")(input)?;
    return Ok((remain, NFA::character(c)));
}

fn dot(input: &str) -> nom::IResult<&str, Box<NFA>> {
    let (remain, _) = char('.')(input)?;
    return Ok((remain, NFA::dot()));
}

fn fold_nfa_states(states: Vec<Box<NFA>>) -> Box<NFA> {
    let nfa = states
        .into_iter()
        .rev()
        .fold(NFA::accept(), |nfa, mut state| {
            state.concatnate(nfa);
            return state;
        });

    let mut head = NFA::head();
    head.concatnate(nfa);

    head
}

trait Concat {
    fn concatnate(&mut self, nfa: Box<NFA>);
}

impl Concat for NFA {
    fn concatnate(&mut self, nfa: Box<NFA>) {
        match self {
            NFA::Head(head) => head.concatnate(nfa),
            NFA::Character(word) => word.concatnate(nfa),
            NFA::Dot(dot) => dot.concatnate(nfa),
            NFA::Choice(choice) => choice.concatnate(nfa),
            NFA::Repeat(repeat) => repeat.concatnate(nfa),
            NFA::Group(group) => group.concatnate(nfa),
            NFA::Accept => {}
        }
    }
}

impl Concat for Head {
    fn concatnate(&mut self, nfa: Box<NFA>) {
        match self.next {
            Some(ref mut next) => next.concatnate(nfa),
            None => self.next = Some(nfa),
        }
    }
}

impl Concat for Character {
    fn concatnate(&mut self, nfa: Box<NFA>) {
        match self.next {
            Some(ref mut next) => next.concatnate(nfa),
            None => self.next = Some(nfa),
        }
    }
}

impl Concat for Dot {
    fn concatnate(&mut self, nfa: Box<NFA>) {
        match self.next {
            Some(ref mut next) => next.concatnate(nfa),
            None => self.next = Some(nfa),
        }
    }
}

impl Concat for Choice {
    fn concatnate(&mut self, nfa: Box<NFA>) {
        match self.next {
            Some(ref mut next) => next.concatnate(nfa),
            None => self.next = Some(nfa),
        }
    }
}

impl Concat for Repeat {
    fn concatnate(&mut self, nfa: Box<NFA>) {
        match self.next {
            Some(ref mut next) => next.concatnate(nfa),
            None => self.next = Some(nfa),
        }
    }
}

impl Concat for Group {
    fn concatnate(&mut self, nfa: Box<NFA>) {
        match self.next {
            Some(ref mut next) => next.concatnate(nfa),
            None => self.next = Some(nfa),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_word_or_dot() {
        let re = Regex::parse("ab.ba").expect("Failed to parse regex");

        let expect = Regex {
            nfa: fold_nfa_states(vec![
                NFA::character('a'),
                NFA::character('b'),
                NFA::dot(),
                NFA::character('b'),
                NFA::character('a'),
            ]),
        };

        assert_eq!(format!("{re:?}"), format!("{expect:?}"));
    }

    #[test]
    fn test_parse_choice_1() {
        let re = Regex::parse("a|b").expect("Failed to parse regex");

        let expect = Regex {
            nfa: fold_nfa_states(vec![NFA::choice(vec![
                fold_nfa_states(vec![NFA::character('a')]),
                fold_nfa_states(vec![NFA::character('b')]),
            ])]),
        };

        assert_eq!(format!("{re:?}"), format!("{expect:?}"));
    }

    #[test]
    fn test_parse_choice_2() {
        let re = Regex::parse("a.|.b").expect("Failed to parse regex");

        let expect = Regex {
            nfa: fold_nfa_states(vec![NFA::choice(vec![
                fold_nfa_states(vec![NFA::character('a'), NFA::dot()]),
                fold_nfa_states(vec![NFA::dot(), NFA::character('b')]),
            ])]),
        };

        assert_eq!(format!("{re:?}"), format!("{expect:?}"));
    }

    #[test]
    fn test_parse_group_1() {
        let re = Regex::parse("(a|b)").expect("Failed to parse regex");

        let expect = Regex {
            nfa: fold_nfa_states(vec![NFA::group(fold_nfa_states(vec![NFA::choice(vec![
                fold_nfa_states(vec![NFA::character('a')]),
                fold_nfa_states(vec![NFA::character('b')]),
            ])]))]),
        };

        assert_eq!(format!("{re:?}"), format!("{expect:?}"));
    }

    #[test]
    fn test_parse_group_2() {
        let re = Regex::parse(".(a.|b).|.(c|.d).").expect("Failed to parse regex");

        let expect = Regex {
            nfa: fold_nfa_states(vec![NFA::choice(vec![
                fold_nfa_states(vec![
                    NFA::dot(),
                    NFA::group(fold_nfa_states(vec![NFA::choice(vec![
                        fold_nfa_states(vec![NFA::character('a'), NFA::dot()]),
                        fold_nfa_states(vec![NFA::character('b')]),
                    ])])),
                    NFA::dot(),
                ]),
                fold_nfa_states(vec![
                    NFA::dot(),
                    NFA::group(fold_nfa_states(vec![NFA::choice(vec![
                        fold_nfa_states(vec![NFA::character('c')]),
                        fold_nfa_states(vec![NFA::dot(), NFA::character('d')]),
                    ])])),
                    NFA::dot(),
                ]),
            ])]),
        };

        assert_eq!(format!("{re:?}"), format!("{expect:?}"));
    }

    #[test]
    fn test_parse_repeat_1() {
        let re = Regex::parse("aa*b").expect("Failed to parse regex");

        let expect = Regex {
            nfa: fold_nfa_states(vec![
                NFA::character('a'),
                NFA::repeat(fold_nfa_states(vec![NFA::character('a')])),
                NFA::character('b'),
            ]),
        };

        assert_eq!(format!("{re:?}"), format!("{expect:?}"));
    }

    #[test]
    fn test_parse_complex_regex_1() {
        let re = Regex::parse("..*(abc|ab.*)*ab|(cba)*").expect("Failed to parse regex");

        let expect = Regex {
            nfa: fold_nfa_states(vec![NFA::choice(vec![
                fold_nfa_states(vec![
                    NFA::dot(),
                    NFA::repeat(fold_nfa_states(vec![NFA::dot()])),
                    NFA::repeat(fold_nfa_states(vec![NFA::group(fold_nfa_states(vec![
                        NFA::choice(vec![
                            fold_nfa_states(vec![
                                NFA::character('a'),
                                NFA::character('b'),
                                NFA::character('c'),
                            ]),
                            fold_nfa_states(vec![
                                NFA::character('a'),
                                NFA::character('b'),
                                NFA::repeat(fold_nfa_states(vec![NFA::dot()])),
                            ]),
                        ]),
                    ]))])),
                    NFA::character('a'),
                    NFA::character('b'),
                ]),
                fold_nfa_states(vec![NFA::repeat(fold_nfa_states(vec![NFA::group(
                    fold_nfa_states(vec![
                        NFA::character('c'),
                        NFA::character('b'),
                        NFA::character('a'),
                    ]),
                )]))]),
            ])]),
        };

        assert_eq!(format!("{re:?}"), format!("{expect:?}"));
    }

    #[test]
    fn test_regex_1() {
        let re = Regex::parse("a*").expect("Failed to parse regex");
        assert_eq!(re.test(""), true, r#"test to """#);
        assert_eq!(re.test("a"), true, r#"test to "a""#);
        assert_eq!(re.test("aaa"), true, r#"test to "aaa""#);
        assert_eq!(re.test(".a"), true, r#"test to ".a""#);
    }

    #[test]
    fn test_regex_2() {
        let re = Regex::parse("x*x").expect("Failed to parse regex");
        assert_eq!(re.test("xxx"), true, r#"test to "xxx""#);
        assert_eq!(re.test("xyx"), true, r#"test to "xyx""#);
        assert_eq!(re.test("yyx"), false, r#"test to "yyx""#);
        assert_eq!(re.test("yyy"), false, r#"test to "yyy""#);
    }

    #[test]
    fn test_regex_3() {
        let re = Regex::parse("aa*|bb*").expect("Failed to parse regex");
        assert_eq!(re.test("a"), true, r#"test to "a""#);
        assert_eq!(re.test("b"), true, r#"test to "b""#);
        assert_eq!(re.test("aaa"), true, r#"test to "aaa""#);
        assert_eq!(re.test("bbb"), true, r#"test to "bbb""#);
        assert_eq!(re.test(""), false, r#"test to """#);
    }

    #[test]
    fn test_regex_4() {
        let re = Regex::parse("x..*").expect("Failed to parse regex");
        assert_eq!(re.test(""), false, r#"test to """#);
        assert_eq!(re.test("x"), false, r#"test to "x""#);
        assert_eq!(re.test("xx"), true, r#"test to "xx""#);
        assert_eq!(re.test("xxx"), true, r#"test to "xxx""#);
        assert_eq!(re.test("xxy"), true, r#"test to "xxy""#);
        assert_eq!(re.test("xxyy"), true, r#"test to "xxyy""#);
        assert_eq!(re.test("xxyyzz"), true, r#"test to "xxyyzz""#);
    }

    #[test]
    fn test_complex_regex_3() {
        let re = Regex::parse(".*x(1(abc)*|2(def)*)x.*").expect("Failed to parse regex");
        assert_eq!(re.test("xxx1abcabcxxx"), true, r#"test to "xxx1abcabcxxx""#);
        assert_eq!(re.test("x2defdefx"), true, r#"test to "x2defdefx""#);
        assert_eq!(
            re.test("xxx1abcdefxxx"),
            false,
            r#"test to "xxx1abcdefxxx""#
        );
        assert_eq!(
            re.test("xxx2defabcxxx"),
            false,
            r#"test to "xxx2defabcxxx""#
        );
    }
}
