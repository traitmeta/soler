use std::env;

#[allow(dead_code)]
pub(crate) fn closures() {
    let doubler = |x| x * 2;
    let value = 5;
    let twice = doubler(value);
    println!("{} doubled is {}", value, twice);

    let big_closure = |b, c| {
        let z = b + c;
        z * twice
    };

    let some_number = big_closure(1, 2);
    println!("Result form closure: {}", some_number)
}

#[allow(dead_code)]
fn basic() {
    let name = env::args().nth(1);
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

    let mut result = "Sanity reigns";
    if 1 == 2 {
        result = "Nothing makes sense";
    };
    print!("Result of computation: {:?}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closure() {
        closures();
        assert_eq!(3, 3);
    }
}
