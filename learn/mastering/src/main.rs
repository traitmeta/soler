#[derive(Debug)]
struct Items(u32);

struct Person(String);
fn main() {
    let items = Items(2);
    let mut a = Items(20);
    // using scope to limit the mutation of `a` within this block by b
    {
        // can take a mutable reference like this too
        let b = &mut a; // same as: let b = &mut a;
        b.0 += 25;
    }
    println!("{:?}", items);
    println!("{:?}", a); // without the above scope this does not compile. Try removing the scope

    let a = Person("Richard Feynman".to_string());
    println!("{} was a great physicist !", a.0);
    let _b = a;
}
