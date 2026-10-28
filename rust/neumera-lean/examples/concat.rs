//! Demonstration: build two vectors in Rust, concatenate them in Lean, read
//! the result back, and show which checked contract the operation relies on.

use neumera_lean::{IntVec, contracts};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = IntVec::from_slice(&[1, 2, 3])?;
    let b = IntVec::from_slice(&[40, 50, 1 << 40])?;
    let c = a.concat(&b);
    println!("a        = {:?}", a.to_vec()?);
    println!("b        = {:?}", b.to_vec()?);
    println!("concat   = {:?}", c.to_vec()?);
    println!("lengths  = {} + {} = {}", a.len(), b.len(), c.len());
    println!();
    println!("relies on contract {}", contracts::CONCAT_LENGTH.name);
    println!("  statement: {}", contracts::CONCAT_LENGTH.statement);
    println!("  export:    {}", contracts::CONCAT_LENGTH.export);
    println!("  axioms:    {:?}", contracts::CONCAT_LENGTH.axioms);
    println!(
        "  checked by Lean {}; library hash {}",
        contracts::LEAN_VERSION,
        contracts::EXECUTABLE_HASH
    );
    Ok(())
}
