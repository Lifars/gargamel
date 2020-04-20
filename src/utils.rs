pub trait Quoted{
    fn quoted(&self) -> String;
}

impl Quoted for str {
    fn quoted(&self) -> String {
        format!("\"{}\"", self)
    }
}
