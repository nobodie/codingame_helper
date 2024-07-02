fn main() -> Result<(), std::io::Error>{
    {{ safe_name }}::main(std::io::stdin().lock(), std::io::stdout())
}
