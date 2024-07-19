use std::borrow::Cow;

#[derive(Debug)]
pub struct Repository<'r> {
    pub name: Cow<'r, str>,
    pub tag: &'r str,
}

impl<'r> Repository<'r> {
    pub fn new(image: &'r str) -> Self {
        if let Some((name, tag)) = image.split_once(':') {
            Self {
                name: Self::parse_repo_name(name),
                tag,
            }
        } else {
            Self {
                name: Self::parse_repo_name(image),
                tag: "latest",
            }
        }
    }

    fn parse_repo_name(name: &'r str) -> Cow<'r, str> {
        match name.contains('/') {
            true => Cow::from(name),
            false => Cow::from(format!("library/{}", name).as_str().to_owned()),
        }
    }
}
