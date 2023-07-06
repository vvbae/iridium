use nom_supreme::error::ErrorTree;

pub type ParseError<'a> = ErrorTree<&'a str>;
