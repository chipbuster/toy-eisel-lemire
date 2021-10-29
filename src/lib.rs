pub fn print_stuff() {
    println!("Hey there!")
}

#[cfg(test)]
pub mod tests{
    #[test]
    pub fn it_works() {
        assert!(true);
    }

    #[test]
    pub fn it_fails(){
        assert!(false);
    }
}
