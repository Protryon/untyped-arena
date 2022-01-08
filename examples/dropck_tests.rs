use untyped_arena::Arena;

struct A<'a> {
    reference: Option<&'a A<'a>>,
    mem: Box<u8>,
}

impl<'a> Default for A<'a> {
    fn default() -> Self {
        Self {
            reference: None,
            mem: Box::new(0),
        }
    }
}

impl<'a> Drop for A<'a> {
    fn drop(&mut self) {
        if let Some(reference) = self.reference {
            // cause a use-after-free
            println!("{}", *reference.mem);
        }
    }
}

// this should not compile to show soundness
fn main() {
    let arena = Arena::new();
    let mut a = arena.alloc(A::default());
    let mut b = arena.alloc(A::default());
    b.reference = Some(&*a);
}
