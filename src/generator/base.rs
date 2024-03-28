pub trait Generator {
    fn new() -> Self;
    fn get_statement() -> String;
    fn get_params() -> Vec<String>;
}