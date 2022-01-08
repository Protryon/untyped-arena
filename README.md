# untyped-arena

untyped-arena provides an Arena allocator implementation that is safe and untyped with minimal complexity

## Usage

```
let arena = Arena::new();
// create our object, and allocate it within `arena`
let my_struct: &mut MyStruct = arena.alloc(MyStruct { ... });
// dropping the arena drops `my_struct`
drop(arena);
// my_struct can no longer be referenced here
```