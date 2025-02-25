use std::fmt;
use std::string::ParseError;
use std::cmp::Ordering::{Less, Equal, Greater};
use std::str::FromStr;

pub enum VersionRelation {
    StrictlyLess, // <<
    LessOrEqual, // <=
    Equal, // =
    GreaterOrEqual, // >=
    StrictlyGreater // >>
}

impl fmt::Display for VersionRelation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            VersionRelation::StrictlyLess =>    write!(f, "<<"),
            VersionRelation::LessOrEqual =>     write!(f, "<="),
            VersionRelation::Equal =>           write!(f, "="),
            VersionRelation::GreaterOrEqual =>  write!(f, ">="),
            VersionRelation::StrictlyGreater => write!(f, ">>")
        }
    }
}

impl FromStr for VersionRelation {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "<<" => Ok(VersionRelation::StrictlyLess),
            "<=" => Ok(VersionRelation::LessOrEqual),
            "=" =>  Ok(VersionRelation::Equal),
            ">=" => Ok(VersionRelation::GreaterOrEqual),
            ">>" => Ok(VersionRelation::StrictlyGreater),
            _ => panic!("bad version relation {}", s) // gah this should be a ParseError but I don't know how to do that, accepting PRs
        }
    }
}


#[derive(PartialEq,Eq)]
pub struct DebianVersionNum {
    epoch : String,
    upstream : String,
    debian : String
}

impl fmt::Display for DebianVersionNum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let epoch_sep = if self.epoch.is_empty() {""} else {":"};
        let deb_sep = if self.debian.is_empty() {""} else {"-"};
        write!(f, "{}{}{}{}{}", self.epoch, epoch_sep, self.upstream, deb_sep, self.debian)
    }
}

impl FromStr for DebianVersionNum {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ep, rest0) = match s.find(':') {
            None => ("", s),
            Some(e) => { let (_e, _r) = s.split_at(e); (_e, &_r[1..]) }
        };
        let (up, deb) = match rest0.rfind('-') {
            None => (rest0, ""),
            Some(d) => { let (_u, _d) = rest0.split_at(d); (_u, &_d[1..]) }
        };
        Ok(DebianVersionNum {
            epoch : ep.to_string(),
            upstream: up.to_string(),
            debian: deb.to_string()
        })
    }
}

impl Ord for DebianVersionNum {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for DebianVersionNum {
    // https://www.debian.org/doc/debian-policy/ch-controlfields.html#version
    // wow this is painful
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        fn extract_nonnum(s: &str) -> (&str,&str) {
            let first_digit = s.find(|c:char| c.is_ascii_digit());
            match first_digit {
                None => (s, ""),
                Some(fd) => {
                    let (nonnum, rest) = s.split_at(fd);
                    return (nonnum, rest);
                }
            }
        }
        fn extract_num(s: &str) -> (&str,&str) {
            let first_nondigit = s.find(|c:char| !c.is_ascii_digit());
            match first_nondigit {
                None => (s, ""),
                Some(fd) => {
                    let (num, rest) = s.split_at(fd);
                    return (num, rest);
                }
            }
        }

        fn to_debian_chars(s: &str) -> Vec<i32> {
            let mut v = vec![];
            // all the letters sort earlier than all the non-letters and so that a tilde sorts before anything, even the end of a part
            // . + - ~
            for c in s.bytes() {
                let cc:i32 = match c {
                    46 /* '.' */ => 256+46,
                    43 /* '+' */ => 256+43,
                    45 /* '-' */ => 256+45,
                    126 /* '~' */ => -1,
                    _ => i32::from(c) };
                v.push(cc);
            }
            v
        }

        fn debian_nonnum_cmp(s: &str, o: &str) -> std::cmp::Ordering {
            let (d_s, d_o) = (to_debian_chars(s), to_debian_chars(o));
            for it in d_s.iter().zip(d_o.iter()) {
                let (c_s, c_o) = it;
                if c_s < c_o { return Less; }
                if c_s > c_o { return Greater; }
            }
            // aa < aaa
            if s.len() < o.len() { return Less; }
            // aa~ < aa
            if s.len() > o.len() && s.ends_with('~') { return Less; }
            // aaa > aa
            if s.len() > o.len() { return Greater; }
            Equal
        }

        fn debian_cmp(self_vers: &str, other_vers: &str) -> std::cmp::Ordering {
            let mut sv = self_vers;
            let mut ov = other_vers;
            loop {
                if sv.is_empty() && ov.is_empty() {
                    return Equal;
                }
                let (self_nonnum, self_rest) = extract_nonnum(sv);
                let (other_nonnum, other_rest) = extract_nonnum(ov);
                match debian_nonnum_cmp(self_nonnum, other_nonnum) {
                    Less => return Less,
                    Greater => return Greater,
                    _ => ()
                }

                let (self_num, self_rest1) = extract_num(self_rest);
                let (other_num, other_rest1) = extract_num(other_rest);

                let (sn_i, on_i) = (
                    match self_num.parse::<i32>() { Err(_) => 0, Ok(e) => e },
                    match other_num.parse::<i32>() { Err(_) => 0, Ok(e) => e });

                if sn_i != on_i {
                    return sn_i.partial_cmp(&on_i).unwrap();
                }
                sv = self_rest1; ov = other_rest1;
            }
        }

        let (epoch, other_epoch) = (
            match self.epoch.parse::<i32>() { Err(_) => 0, Ok(e) => e },
            match other.epoch.parse::<i32>() { Err(_) => 0, Ok(e) => e });
        if epoch != other_epoch {
            return Some(epoch.partial_cmp(&other_epoch).unwrap());
        }
        let ups = debian_cmp(&self.upstream, &other.upstream);
        if ups != Equal {
            return Some(ups);
        }
        return Some(debian_cmp(&self.debian, &other.debian));
    }
}

pub fn cmp_debversion_with_op(op:&VersionRelation, first: &DebianVersionNum, second: &DebianVersionNum) -> bool {
    return match op {
        VersionRelation::StrictlyLess => first < second,
        VersionRelation::LessOrEqual => first <= second,
        VersionRelation::Equal => first == second,
        VersionRelation::GreaterOrEqual => first >= second,
        VersionRelation::StrictlyGreater => first > second
    }
}
