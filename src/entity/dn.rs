use std::fmt::Display;

use super::schema::Oid;

#[derive(Debug, Default, Clone)]
pub struct Rdn(Vec<(Oid, String)>);

impl Rdn {
    pub fn get(&self) -> &[(Oid, String)] {
        self.0.as_slice()
    }
}

impl<O, S> From<Vec<(O, S)>> for Rdn
where
    O: Into<Oid> + Clone,
    S: Into<String> + Clone,
{
    fn from(value: Vec<(O, S)>) -> Self {
        let vals = value
            .into_iter()
            .map(|(o, s)| (o.into(), s.into()))
            .collect();
        Rdn(vals)
    }
}

impl<O, S> From<(O, S)> for Rdn
where
    O: Into<Oid>,
    S: Into<String>,
{
    fn from(value: (O, S)) -> Self {
        let rdn = (value.0.into(), value.1.into());
        Rdn(vec![rdn])
    }
}

impl<'a> IntoIterator for &'a Rdn {
    type Item = &'a (Oid, String);

    type IntoIter = std::slice::Iter<'a, (Oid, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Display for Rdn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = self.0.iter().fold(String::new(), |acc, (oid, val)| {
            format!("{}{}={}+", acc, oid, val)
        });

        // pop the trailing +
        s.pop();

        write!(f, "{}", s)
    }
}

#[derive(Debug, Default, Clone)]
pub struct DN {
    rdns: Vec<Rdn>,
}

impl DN {
    pub fn new(rdns: Vec<Rdn>) -> DN {
        DN { rdns }
    }

    pub fn append(&mut self, rdn: Rdn) {
        self.rdns.push(rdn)
    }

    pub fn first(&self) -> Option<&Rdn> {
        self.rdns.first()
    }

    pub fn as_slice(&self) -> &[Rdn] {
        self.rdns.as_slice()
    }

    pub fn parent_dn(&self) -> DN {
        DN {
            rdns: self.rdns.iter().map(|rdn| rdn.to_owned()).skip(1).collect(),
        }
    }
}

impl<'a> IntoIterator for &'a DN {
    type Item = &'a Rdn;
    type IntoIter = std::slice::Iter<'a, Rdn>;

    fn into_iter(self) -> Self::IntoIter {
        self.rdns.iter()
    }
}

impl Display for DN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = self
            .rdns
            .iter()
            .fold(String::new(), |acc, rdn| format!("{}{},", acc, rdn));

        // pop the trailing comma
        s.pop();

        write!(f, "{}", s)
    }
}

#[test]
fn test_display_dn() {
    let dn = DN::new(vec![
        Rdn::from(("cn-oid", "Test")),
        Rdn::from(vec![("ou-oid", "Test"), ("cn-oid", "Test OU")]),
        Rdn::from(("dc-oid", "dev")),
    ]);

    let expected = "cn-oid=Test,ou-oid=Test+cn-oid=Test OU,dc-oid=dev";
    assert_eq!(dn.to_string(), expected)
}
