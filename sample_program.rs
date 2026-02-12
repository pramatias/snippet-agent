#[cfg(test)]
mod tests {
    #[test]
    fn test_json_selectors_ast_grep() {
        use super::*;
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union IntOrFloat {
    pub i: u32,
    pub f: f32,
}

enum Result<T, E>
where
    E: std::error::Error,
{
    Ok(T),
    Err(E),
}

struct MyType;
struct Wrapper<T>(T);
struct Array<T, const N: usize>([T; N]);

trait SomeTrait<T> {
    fn my_function();
}
trait SomeTrait { type Assoc; }
trait SomeTrait { fn trait_function(t: T) -> Self { Wrapper(t) }}
trait TraitToImpl {}

impl<T> SomeTrait for Wrapper<T> {
    type Assoc = Wrapper<T>;
}
impl MyType {
    fn inherent_method(&self) {}
}

// Inherent impl with generics (lifetimes, type params, const generics) + where
impl<'a, T: Clone, const N: usize> Array<T, N>
where
    T: std::fmt::Debug + Deserialize,
{
    fn new_from_slice(_s: &'a [T]) -> Self { unimplemented!() }
}

// Trait impl (normal)
impl std::fmt::Display for MyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyType")
    }
}

// Trait impl with generics and where clause
impl<T> From<T> for Wrapper<T>
where
    T: Sized,
{
    fn from(t: T) -> Self { Wrapper(t) }
}

// ==== 5) unsafe impl (for an unsafe trait or to assert invariants)
unsafe impl Send for Wrapper<u8> {}

// ==== 6) negative impl (unstable / feature-gated — may not parse on stable toolchains)
// impl !Send for NotSendType {} // <-- example of negative impl (commented out if your parser rejects it)

// ==== 7) impl for a qualified/associated type (<T as Trait>::Assoc)
impl<T> TraitToImpl for <T as SomeTrait>::Assoc
where
    T: SomeTrait,
{
    // empty body allowed
    fn iterator() -> impl Iterator<Item = u8> {
        std::iter::once(1u8)
    }
}

// Impl-trait type in return position (NOT an impl block)
fn make_iter() -> impl Iterator<Item = u8> {
    std::iter::once(1u8)
}

// 1) Simple alias (module-level)
type Id = u32;
// usage: let x: Id = 5u32;

// 2) Visibility on an alias
pub(crate) type PublicId = u32;

// 3) Generic alias
type Wrap<T> = Wrapper<T>;
// usage: let w: Wrap<i32> = Wrapper(42);

// 4) Generic alias with a trait bound on the parameter
type DebugVec<T: std::fmt::Debug> = Vec<T>;
// enforces T: Debug at the alias-site

// 5) Generic alias with a default type parameter
type MyBytes<T = u8> = Vec<T>;
// usage: let b: MyBytes = Vec::<u8>::new();

// 6) Const-generic alias (arrays)
type ArrayN<T, const N: usize> = [T; N];
// usage: let a: ArrayN<i32, 4> = [0; 4];

// 7) Lifetime in an alias (references / slices)
type BytesRef<'a> = &'a [u8];
// usage: fn parse(b: BytesRef) { /* ... */ }

// 8) Alias to an associated type of a trait implementation
type AssocOf<T: SomeTrait> = <T as SomeTrait>::Assoc;
// usage: let _v: AssocOf<Wrapper<i32>> = Wrapper(0);

// 9) Alias for Result with a default error type
type MyResult<T, E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;
// usage: fn foo() -> MyResult<()> { Ok(()) }

// 10) Alias for a function pointer (and extern "C" variant)
type Handler = fn(i32) -> i32;
type ExternHandler = extern "C" fn(i32) -> i32;
// usage: fn double(x: i32) -> i32 { x * 2 } let h: Handler = double;

// 11) Alias for a trait object (boxed)
type BoxedFn = Box<dyn Fn(i32) -> i32 + Send + 'static>;
// usage: let f: BoxedFn = Box::new(|x| x + 1);

// 12) Alias to an unsafe/union type, or to any existing type
type IOrFAlias = IntOrFloat;
// usage: let u: IOrFAlias = IntOrFloat { i: 0 };

// 13) Alias for raw pointer types
type RawPtr<T> = *mut T;
// usage: let p: RawPtr<u8> = std::ptr::null_mut();

// 14) Alias for complex composed types
type BoxedIter<'a, T> = Box<dyn Iterator<Item = T> + 'a>;
// usage: fn iter<'a>() -> BoxedIter<'a, u8> { Box::new(std::iter::once(1u8)) }
