//! This is a CLI to turn text into
//! SVG diagrams using the badascii backend
#![doc = my_macro!("
                                                                             
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
use badascii_doc::my_macro;

fn main() {
    let foo = my_macro!(
        "
    +-----+
    |     |
    +-----+
    
    "
    );
    println!("{}", foo);

    let svg = my_macro!(
        "                                                                         
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
                                    
   "
    );
    std::fs::write("test.svg", svg).unwrap();
}
