pub mod term;
use term::*;


/*


data Nat {
    zer: Nat,
    suc{pred: Nat}: Nat
}
match x : Nat to Bool {
    zer: true
    suc{pred: x}: false
}
foo = data Fin : (n : Nat) -> Set {
    zer : (n : Nat) -> Fin (suc n),
    suc : (n : Nat, i : Fin n) -> Fin (suc n)
}
$Fin @n Nat *
    |zer @n Nat :Fin :suc n
    |suc @n Nat @i :Fin n :Fin :suc n
/suc %suc Nat
/zer %zer Nat

data Nat : Type
| succ(x : Nat) : Nat
| zero          : Nat

data Bool : Type
| true  : Bool
| false : Bool

data Fin : (n : Nat) -> Type
| zer : (n : Nat) -> Fin (suc n)
| suc : (n : Nat, i : Fin n) -> Fin (suc n)

...
.true Bool : (data Bool : Type ...)
.zer Fin : (n : Nat) -> (data Fin ...) (suc n)

def cas
    ( n : Nat
    , P : Type
    , s : (n : Nat) -> P
    , z : P)
    case Nat n to P
    |succ(x) s(x)
    |zero    z

def ind
    ( n : Nat
    , P : Nat -> Type
    , s : (n : Nat) -> P n -> P (Nat.succ n)
    , z : P(Nat.zero))
    case Nat n to P(self)
    |succ(x) s(x, induce(x)) -- for any x:TYPE, ind(x) -> P(x)
    |zero    z

    (ind : (x : TYPE) -> P x) -> (x : TYPE) -> P x
    
case Nat x to Bool
ind:
    #b @ P Bool
    #t :B tru
    #t :B fal
    %x Bool :P _
    |tru t
    |fal f
    

...

def is_zero(a : Nat)
    case Nat a to Bool
    | succ(pred) false
    | zero       true

def add(a : Nat, b : Nat)
    fold Nat a to Nat
    | succ(add_a) (b : Nat) -> Suc(add_a(b))
    | zero        (b : Nat) -> Zer

*/

fn main() -> Result<(), std::string::String> {
    let term : Term = from_string(b"
        /Nat $Nat *
            |suc @x Nat Nat
            |zer Nat
        
        .suc Nat






        /Bool $Bool *
            |tru Bool
            |fal Bool
        /tru .tru Bool
        /fal .fal Bool

        /T #P * #t P #f P f
        /F #P * #t P #f P f

        ~Bool tru Bool |tru T |fal F


        :.suc Nat
         .zer Nat

    ");

    //pub fn make_case_fun(fun : &Term, idt : &Term, ret : &Term, mut val : Term) -> Term {







    println!("term {}", term);

    let mut norm = term.clone();
    reduce(&mut norm);
    println!("norm {}", norm);

    match infer(&term) {
        Ok(typ) => println!("type: {}", typ),
        Err(err) => println!("type error: {}", err)
    }

    Ok(())
}












































