pub mod term;
use term::*;

pub fn get(name : &[u8], defs : &Defs) -> Term {
    defs.get(&name.to_vec())
        .expect("Attempted to run unexisting program.")
        .clone()
        .1
}

pub fn nf(name : &[u8], defs : &Defs) -> Term {
    let mut val = get(name, defs);
    term::reduce(&mut val, &defs);
    val
}

pub fn ty(name : &[u8], defs : &Defs) -> Result<Term, std::string::String> {
    let val = get(name, defs);
    term::infer(&val, &defs)
}

fn main() -> Result<(), std::string::String> {
    let defs : Defs = build(b"
        =id   * #P * #x P x
        =zer  * #P * #S @ P P #Z P Z
        =Bits * @P * @O @ * P #I @ * P #E P P
        =O    * #xs * #P * #O @ * P #I @ * P #E P :O xs
        =I    * #xs * #P * #O @ * P #I @ * P #E P :I xs
        =E    * #P * #O @ * P #I @ * P #E P E
        =list * :I :O list
        =tail * #xs * ::::xs * #xs * xs #xs * xs E
        =fst  * #xs * ::::xs *
            #xs * #a * #b * a
            #xs * #a * #b * b
            #x * x
        =main * id
    ");

    Nat
        = ∀ (P : *)
        -> (Nat -> P)
        -> P
        -> P

    type(Nat) =
        if type(Nat) == Set
            then Set
            else Err

    foo : Bool; foo = if foo then true else false





    println!("norm {}", nf(b"main", &defs));

    println!("type {}", ty(b"main", &defs)?);

    Ok(())
}
