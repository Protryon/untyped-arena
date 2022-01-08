use untyped_arena::Arena;

struct DropTester {
    teller: *mut bool,
}

impl DropTester {
    pub fn new(out: &mut bool) -> Self {
        Self {
            teller: out as *mut bool,
        }
    }
}

impl Drop for DropTester {
    fn drop(&mut self) {
        *unsafe { self.teller.as_mut().unwrap() } = true;
    }
}

#[test]
fn test_drop_basic() {
    let arena = Arena::new();

    let mut out = false;
    let tester = DropTester::new(&mut out);
    arena.alloc(tester);

    drop(arena);

    assert!(out);
}

#[test]
fn test_drop_many() {
    let arena = Arena::new();

    const DEPTH: usize = 10000;

    let mut out = [false; DEPTH + 1];
    for i in 0..DEPTH {
        arena.alloc(DropTester::new(&mut out[i]));
    }

    drop(arena);

    for i in 0..DEPTH {
        assert!(out[i]);
    }
    assert!(!out[DEPTH]);
}
