use crate::error::AppError;
use std::io::{self, Write};

pub fn read_line_safe() -> Result<String, AppError> 
{
    let mut input = String::new();
    // Handles IO error returned by read_line
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Generic function to read and parse any type T that implements FromStr (like i32)
pub fn read_numeric_input<T>(prompt: &str) -> Result<T, AppError>
where
    T: std::str::FromStr, // T must be parseable from a string
    <T as std::str::FromStr>::Err: std::fmt::Debug, // T's parse error must be debuggable
{
    loop 
    {
        println!("{}", prompt);
        let input_str = read_line_safe()?;

        match input_str.parse::<T>() 
        {
            Ok(val) => return Ok(val),
            Err(_) => 
            {
                println!("Invalid input. Please enter a valid number.");
                // Note: The original Error::Parse is only returned if an IO error occurs above.
            }
        }
    }
}

pub fn read_yes_no(prompt: &str) -> Result<bool, AppError> 
{
    loop 
    {
        println!("{} (y/n):", prompt);
        let input = read_line_safe()?.to_lowercase(); // Handle case here
        match input.as_str() 
        {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Please type 'y' or 'n'."),
        }
    }
}