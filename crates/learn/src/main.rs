#[derive(Debug)]
struct Items(u32);

struct Person(String);
fn main() {
    let items = Items(2);
    let items_ptr = &items;
    let ref items_ref = items;
    assert_eq!(items_ptr as *const Items, items_ref as *const Items);
    let mut a = Items(20);
    // using scope to limit the mutation of `a` within this block by b
    {
        // can take a mutable reference like this too
        let ref mut b = a; // same as: let b = &mut a;
        b.0 += 25;
    }
    println!("{:?}", items);
    println!("{:?}", a); // without the above scope this does not compile. Try removing the scope

    let a = Person("Richard Feynman".to_string());
    match a {
        Person(ref name) => println!("{} was a great physicist !", name),
        _ => panic!("Oh no !"),
    }
    let b = a;
}
