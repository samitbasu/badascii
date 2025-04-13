//! This is a CLI to turn text into
//! SVG diagrams using the badascii backend
#![doc = badascii_smooth!("
                                                                             
     +---------------------+                                               
     |                     |                                               
+--->| data           data |o--+                                           
|    |                     |   |                                           
|   o| full           next |>  |                                           
v    |                     |   |                                           
    o| overflow  underflow |o--+                                           
     |                     |                                               
     +---------------------+                                               
                                                                          
                                                                          
                                                                          
                                                                          
                                              +---------------------+     
                                              |                     |     
                                         +--->| data           data |o--+ 
                                         |    |                     |   | 
                                         |   o| full           next |>  | 
                                         v    |                     |   | 
                                             o| overflow  underflow |o--+ 
     +---------------------+                  |                     |     
     |                     |                  +---------------------+     
+--->| data           data |o--+                                          
|    |                     |   |                                          
|   o| full           next |>  |                                          
v    |                     |   |                                          
    o| overflow  underflow |o--+                                          
     |                     |                                              
     +---------------------+                                              
                                     
    ")]
//! More examples below.
use badascii_doc::{badascii, badascii_smooth};

use clap::Parser;
use clap_stdin::FileOrStdin;

#[derive(Debug, Parser)]
struct Args {
    #[arg(default_value = "-")]
    input: FileOrStdin,
}

fn main() -> Result<(), clap_stdin::StdinError> {
    let args = Args::parse();
    println!("input = {}", args.input.contents()?);
    Ok(())
}
