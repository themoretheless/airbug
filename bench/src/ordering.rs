//! Deterministic natural ordering for case paths and numeric argument labels.
use std::cmp::Ordering;

enum Number {
    NegativeInfinity,
    Finite(Decimal),
    PositiveInfinity,
}
impl Number {
    fn parse(text: &str) -> Option<Self> {
        // Match Rust's floating-point infinity spellings without converting finite
        // decimal labels to f64 (which would lose integer precision).
        let unsigned = text.strip_prefix(['-', '+']).unwrap_or(text);
        if unsigned.eq_ignore_ascii_case("inf") || unsigned.eq_ignore_ascii_case("infinity") {
            return Some(if text.starts_with('-') {
                Self::NegativeInfinity
            } else {
                Self::PositiveInfinity
            });
        }
        Decimal::parse(text).map(Self::Finite)
    }
    fn compare(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Finite(a), Self::Finite(b)) => a.compare(b),
            (Self::NegativeInfinity, Self::NegativeInfinity)
            | (Self::PositiveInfinity, Self::PositiveInfinity) => Ordering::Equal,
            (Self::NegativeInfinity, _) | (_, Self::PositiveInfinity) => Ordering::Less,
            (Self::PositiveInfinity, _) | (_, Self::NegativeInfinity) => Ordering::Greater,
        }
    }
}

struct Decimal {
    sign: i8,
    digits: Vec<u8>,
    magnitude: i128,
}
impl Decimal {
    fn parse(text: &str) -> Option<Self> {
        let (negative, text) = if let Some(t) = text.strip_prefix('-') {
            (true, t)
        } else {
            (false, text.strip_prefix('+').unwrap_or(text))
        };
        let (mantissa, exponent) = match text.split_once(['e', 'E']) {
            Some((m, e)) => (m, e.parse::<i64>().ok()? as i128),
            None => (text, 0),
        };
        let mut digits = Vec::new();
        let mut fractional = 0i128;
        let mut dot = false;
        for byte in mantissa.bytes() {
            if byte == b'.' && !dot {
                dot = true;
            } else if byte.is_ascii_digit() {
                digits.push(byte);
                fractional += i128::from(dot);
            } else {
                return None;
            }
        }
        if digits.is_empty() {
            return None;
        }
        let leading = digits.iter().take_while(|d| **d == b'0').count();
        digits.drain(..leading);
        let sign = if digits.is_empty() {
            0
        } else if negative {
            -1
        } else {
            1
        };
        let magnitude = exponent - fractional + digits.len() as i128;
        Some(Self {
            sign,
            digits,
            magnitude,
        })
    }
    fn compare(&self, other: &Self) -> Ordering {
        let sign = self.sign.cmp(&other.sign);
        if sign != Ordering::Equal || self.sign == 0 {
            return sign;
        }
        let magnitude = self.magnitude.cmp(&other.magnitude).then_with(|| {
            (0..self.digits.len().max(other.digits.len()))
                .map(|i| {
                    self.digits
                        .get(i)
                        .unwrap_or(&b'0')
                        .cmp(other.digits.get(i).unwrap_or(&b'0'))
                })
                .find(|o| *o != Ordering::Equal)
                .unwrap_or(Ordering::Equal)
        });
        if self.sign < 0 {
            magnitude.reverse()
        } else {
            magnitude
        }
    }
}

pub(crate) fn natural_path_cmp(a: &str, b: &str) -> Ordering {
    let mut left = a.split('/');
    let mut right = b.split('/');
    loop {
        let order = match (left.next(), right.next()) {
            (Some(x), Some(y)) => match (
                Number::parse(if x.starts_with("const=") && y.starts_with("const=") {
                    &x[6..]
                } else {
                    x
                }),
                Number::parse(if x.starts_with("const=") && y.starts_with("const=") {
                    &y[6..]
                } else {
                    y
                }),
            ) {
                (Some(xn), Some(yn)) => xn.compare(&yn).then_with(|| x.cmp(y)),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => natural_text_cmp(x, y),
            },
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
        };
        if order != Ordering::Equal {
            return order;
        }
    }
}

/// Compare ASCII digit runs by magnitude without integer overflow; ties use spelling.
fn natural_text_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let (mut left, mut right) = (a.as_bytes(), b.as_bytes());
    while !left.is_empty() && !right.is_empty() {
        if left[0].is_ascii_digit() && right[0].is_ascii_digit() {
            let n = left.iter().take_while(|c| c.is_ascii_digit()).count();
            let m = right.iter().take_while(|c| c.is_ascii_digit()).count();
            let x = &left[..n];
            let y = &right[..m];
            let x = &x[x.iter().take_while(|c| **c == b'0').count()..];
            let y = &y[y.iter().take_while(|c| **c == b'0').count()..];
            let order = x.len().cmp(&y.len()).then_with(|| x.cmp(y));
            if order != Ordering::Equal {
                return order;
            }
            left = &left[n..];
            right = &right[m..];
        } else {
            let order = left[0].cmp(&right[0]);
            if order != Ordering::Equal {
                return order;
            }
            left = &left[1..];
            right = &right[1..];
        }
    }
    left.len().cmp(&right.len()).then_with(|| a.cmp(b))
}

/// Compare explicit group membership, never treating slashes inside case labels
/// as group boundaries. Leaves precede child groups at every level.
pub(crate) fn kind_cmp(
    ag: &str,
    af: Option<&str>,
    an: &str,
    bg: &str,
    bf: Option<&str>,
    bn: &str,
) -> Ordering {
    hierarchy_cmp((ag, af, an), (bg, bf, bn), crate::SortOrder::Kind)
}

/// Sort actual group nodes before descending into them. A slash in a leaf's
/// display label does not create another group.
pub(crate) fn hierarchy_cmp(
    (ag, af, an): (&str, Option<&str>, &str),
    (bg, bf, bn): (&str, Option<&str>, &str),
    order: crate::SortOrder,
) -> Ordering {
    let mut a = ag
        .split('/')
        .map(|name| (1u8, name))
        .chain(af.map(|name| (1u8, name)))
        .chain(std::iter::once((0, an)));
    let mut b = bg
        .split('/')
        .map(|name| (1u8, name))
        .chain(bf.map(|name| (1u8, name)))
        .chain(std::iter::once((0, bn)));
    loop {
        match (a.next(), b.next()) {
            (Some((ak, av)), Some((bk, bv))) => {
                let names = if order == crate::SortOrder::Lexical {
                    av.cmp(bv)
                } else {
                    natural_path_cmp(av, bv)
                };
                let compared = if order == crate::SortOrder::Kind {
                    ak.cmp(&bk).then(names)
                } else {
                    names.then_with(|| ak.cmp(&bk))
                };
                if compared != Ordering::Equal {
                    return compared;
                }
            }
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn infinities_sort_outside_finite_numbers_without_rounding() {
        let mut labels = vec![
            "inf",
            "-1e999",
            "-inf",
            "9007199254740993",
            "1e999",
            "9007199254740992",
            "0",
        ];
        labels.sort_by(|a, b| natural_path_cmp(a, b));
        assert_eq!(
            labels,
            [
                "-inf",
                "-1e999",
                "0",
                "9007199254740992",
                "9007199254740993",
                "1e999",
                "inf"
            ]
        );
        for spelling in ["inf", "+INF", "Infinity", "-infinity", "-Inf"] {
            assert!(spelling.parse::<f64>().unwrap().is_infinite());
            let number = Number::parse(spelling).unwrap();
            assert_eq!(
                number.compare(&Number::parse("0").unwrap()),
                if spelling.starts_with('-') {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            );
        }
        assert!(Number::parse("NaN").is_none());
        assert!(Number::parse("infinite").is_none());
        assert_eq!(natural_path_cmp("case/-inf", "case/-10"), Ordering::Less);
    }

    #[test]
    fn numeric_labels_compare_exactly_without_float_rounding() {
        let mut values = vec![
            "9007199254740993",
            "1e-100",
            "-0.02",
            "-10",
            "2.0",
            "-2",
            "0",
            "1.1",
            "9007199254740992",
            "1e100",
            "1.01",
        ];
        values.sort_by(|a, b| natural_path_cmp(a, b));
        assert_eq!(
            values,
            [
                "-10",
                "-2",
                "-0.02",
                "0",
                "1e-100",
                "1.01",
                "1.1",
                "2.0",
                "9007199254740992",
                "9007199254740993",
                "1e100"
            ]
        );
        assert_eq!(
            Decimal::parse("10.00e-1")
                .unwrap()
                .compare(&Decimal::parse("1").unwrap()),
            Ordering::Equal
        );
        assert_eq!(
            Decimal::parse("-0e999")
                .unwrap()
                .compare(&Decimal::parse("0").unwrap()),
            Ordering::Equal
        );
    }
    #[test]
    fn mixed_paths_form_a_total_order() {
        let values = [
            "s/const=-10",
            "s/const=-2",
            "s/const=0",
            "s/const=2",
            "s/const=10",
            "s/const=NaN",
            "s/const=a",
            "s/const=/x",
            "s/-10",
            "s/-2",
            "s/0",
            "s/00",
            "s/1.0",
            "s/1e0",
            "s/1",
            "s/1/x",
            "s/NaN",
            "s/-inf",
            "s/inf",
            "s/+INFINITY",
            "s/-Infinity",
            "s/.2",
            "s/2x",
            "s/10x",
            "s/тест2",
            "s/тест10",
            "s/1e999999999999999999999",
            "s/+",
            "s/.",
        ];
        for a in values {
            for b in values {
                assert_eq!(natural_path_cmp(a, b), natural_path_cmp(b, a).reverse());
                for c in values {
                    if natural_path_cmp(a, b).is_le() && natural_path_cmp(b, c).is_le() {
                        assert!(natural_path_cmp(a, c).is_le(), "{a} <= {b} <= {c}");
                    }
                }
            }
        }
    }
}
