use super::*;
pub trait Attribute<'re> {
    fn new_in(arena: &'re Bump) -> Self;
    fn render(&self) -> Option<HtmlAttribute<'re>>;
}
#[derive(Clone)]
pub struct Classes<'re>(pub Vec<'re, &'re str>);
impl<'re> Attribute<'re> for Classes<'re> {
    fn new_in(arena: &'re Bump) -> Self {
        Self(Vec::new_in(arena))
    }
    fn render(&self) -> Option<HtmlAttribute<'re>> {
        if self.0.is_empty() {
            None
        } else {
            Some(HtmlAttribute {
                name: "class",
                value: HtmlValue::String(self.0.bump().alloc_str(&self.0.join(" "))),
            })
        }
    }
}
impl<'re> Classes<'re> {
    pub fn add(&mut self, class: &str) {
        assert!(!self.0.contains(&class));
        assert!(class.chars().all(|c| !c.is_ascii_whitespace()));
        assert!(!class.is_empty());
        self.0.push(self.0.bump().alloc_str(class));
    }
}
