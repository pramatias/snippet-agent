// ==== types & traits used by the examples ====
struct MyType;
struct Wrapper<T>(T);
struct NotSendType;
struct Array<T, const N: usize>([T; N]);

trait SomeTrait { type Assoc; }
trait TraitToImpl {}
impl<T> SomeTrait for Wrapper<T> { type Assoc = Wrapper<T>; }

// ==== 1) simple inherent impl
impl MyType {
    fn inherent_method(&self) {}
}

// ==== 2) inherent impl with generics (lifetimes, type params, const generics) + where
impl<'a, T: Clone, const N: usize> Array<T, N>
where
    T: std::fmt::Debug + Deserialize,
{
    fn new_from_slice(_s: &'a [T]) -> Self { unimplemented!() }
}

// ==== 3) trait impl (normal)
impl std::fmt::Display for MyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyType")
    }
}

// ==== 4) trait impl with generics and where clause
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
    fn make_iter() -> impl Iterator<Item = u8> {
        std::iter::once(1u8)
    }
}

// ==== 8) impl-trait type in return position (NOT an impl block)
fn make_iter() -> impl Iterator<Item = u8> {
    std::iter::once(1u8)
}

// ==== 9) impl-trait type with additional auto trait bounds
fn make_sendable_iter() -> impl Iterator<Item = u8> + Send {
    std::iter::empty()
}

// ==== 10) impl-trait in argument position
fn takes_asref(x: impl AsRef<str>) {
    let _ = x.as_ref();
}
