use std::env;

#[warn(dead_code)]
pub(crate) fn closures(){
    let doubler = |x| x *2;
    let value  = 5;
    let twice =  doubler(value);
    println!("{} doubled is {}", value,twice);

    let big_closure = |b,c|{
        let z = b+c;
        z*twice
    };

    let some_number = big_closure(1,2);
    println!("Result form closure: {}", some_number)
}

#[warn(dead_code)]
fn basic() {
    let name = env::args().skip(1).next();
    match name {
        Some(n) => println!("Hi there ! {}", n),
        None => panic!("Didn't reveive any name ?"),
    }

    let doubler = |x| x * 2;
    let value = 5;
    let twice = doubler(value);
    println!("{} doubled is {}", value, twice);

    let big_closure = |b, c| {
        let z = b + c;
        z * twice
    };

    let some_number = big_closure(1, 2);
    println!("Result form closure: {}", some_number);

    let result = if 1 == 2 {
        "Nothing makes sense";
    } else {
        "Sanity reigns";
    };
    print!("Result of computation: {:?}", result);
}


#[derive(Debug)]
struct Items(u32);

fn assert_items() {
    let items = Items(2);
    let items_ptr = &items;
    let ref items_ref = items;
    assert_eq!(items_ptr as *const Items, items_ref as *const Items);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closure(){
        closures();
        assert_eq!(3, 3);
    }
}