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

#[cfg(test)]
fn test() {
    #[test]
    fn test_closure(){
        closure()
    }
}