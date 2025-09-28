
use super::*;
impl Renderable for &str {
    fn render(&self, cx: &mut Context) {
        writeln!(cx.output, "{}{self}", cx.indentation).unwrap();
    }
}
impl Renderable for &mut str {
    fn render(&self, cx: &mut Context) {
        writeln!(cx.output, "{}{self}", cx.indentation).unwrap();
    }
}
impl FlowContent for &str {}
impl PhrasingContent for &str {}
impl FlowContent for &mut str {}
impl PhrasingContent for &mut str {}
