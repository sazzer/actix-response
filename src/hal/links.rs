use serde::Serialize;

/// Representation of a single HAL Link.
#[derive(Debug, Serialize, Default, Clone)]
pub struct Link {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// Representation of a set of 1 or more HAL Links.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Links {
    Single(Link),
    Multiple(Vec<Link>),
}

impl Links {
    pub fn push(self, new: Link) -> Self {
        match self {
            Links::Single(first) => Self::Multiple(vec![first, new]),
            Links::Multiple(mut previous) => {
                previous.push(new);
                Self::Multiple(previous)
            },
        }
    }
}

impl<S> From<S> for Link
where
    S: Into<String>,
{
    fn from(href: S) -> Self {
        Self {
            href: href.into(),
            ..Link::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use assert2::{check, let_assert};

    use super::*;

    #[test]
    fn append_to_single() {
        let first = Links::Single("/first".into());
        let updated = first.push("/second".into());

        let_assert!(Links::Multiple(links) = updated);
        check!(links.len() == 2);
        check!(links[0].href == "/first");
        check!(links[1].href == "/second");
    }

    #[test]
    fn append_to_multiple() {
        let first = Links::Multiple(vec!["/first".into(), "/second".into()]);
        let updated = first.push("/third".into());

        let_assert!(Links::Multiple(links) = updated);
        check!(links.len() == 3);
        check!(links[0].href == "/first");
        check!(links[1].href == "/second");
        check!(links[2].href == "/third");
    }
}
