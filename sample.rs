struct Person<'a> {
    name: &'a str,
}

#[allow(unused_mut)]
#[allow(unused_variables)]
fn main() {
    let mut x: i32 = 0;
    let y: i64 = 0;
    let str: &str = "Hello";
    let strr: String = "Oyyyy".to_string();
    let boo: bool = false;
    const BO: bool = true;
    println!("Hello, Crowbar! x = {}", x);
    println!("Hello, Crowbar! y = {}", y);
    println!("{}", str);
    println!("{}", strr);
    println!("{}", boo);
    println!("{}", BO);

    let person = Person {
        name: strr.as_str(),
    };
    println!("{:?}", person.name);
}
