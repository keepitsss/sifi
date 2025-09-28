use super::*;
pub type Hook<'re, This> = &'re dyn Fn(&This, &mut Context);
pub trait PreRenderHooks<'re>: Sized
where
    Self: 're,
{
    type This;
    /// # Safety
    /// This will overwrite previous hook
    unsafe fn set_pre_render_hook(&mut self, hook: impl Fn(&Self::This, &mut Context) + 're);

    fn get_pre_render_hook(&self) -> Option<Hook<'re, Self::This>>;

    fn with_pre_render_hook(mut self, new_hook: impl Fn(&Self::This, &mut Context) + 're) -> Self {
        self.add_pre_render_hook(new_hook);
        self
    }
    fn add_pre_render_hook(&mut self, new_hook: impl Fn(&Self::This, &mut Context) + 're) {
        match self.get_pre_render_hook() {
            Some(prev_hook) => unsafe {
                self.set_pre_render_hook(move |this, cx| {
                    prev_hook(this, cx);
                    new_hook(this, cx);
                });
            },
            None => unsafe {
                self.set_pre_render_hook(new_hook);
            },
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PreRenderHookStorage<'re, This> {
    hook: Option<Hook<'re, This>>,
    arena: &'re Bump,
}
impl<'re, T> PreRenderHookStorage<'re, T> {
    pub(crate) fn new_in(arena: &'re Bump) -> Self {
        PreRenderHookStorage { hook: None, arena }
    }
}
impl<'re, This> PreRenderHooks<'re> for PreRenderHookStorage<'re, This> {
    type This = This;
    fn get_pre_render_hook(&self) -> Option<Hook<'re, This>> {
        self.hook
    }
    unsafe fn set_pre_render_hook(&mut self, hook: impl Fn(&This, &mut Context) + 're) {
        let hook = self.arena.alloc(hook) as Hook<'re, This>;
        self.hook = Some(self.arena.alloc(|this: &This, cx: &mut Context| {
            hook(this, cx);
        }));
    }
}
