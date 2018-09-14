extern crate absal;

pub mod term;


fn main() -> Result<(), &'static str> {
    let term = term::from_string(b"
        /Bool
            @P *
            @t P
            @f P
            P

        /fal
            #P *
            #T P
            #F P
            F

        /tru
            #P *
            #T P
            #F P
            T

        /B
            #b Bool
            @P @b Bool *
            @M @b Bool :P b
            :P b

        /not
            #a Bool
            :::a Bool fal tru

        /Nat
            @P *
            @s @ P P
            @z P
            P

        /suc
            #n Nat
            #P *
            #s @ P P
            #z P
            :::n P s z

        /zer
            #P *
            #s @ P P
            #z P
            z

        /MkB
            #b Bool
            #P @b Bool *
            #M @b Bool :P b
            :M b

        /fst
            #A * 
            #B @ A *
            #s $a A :B a
            %s A x y x

        /snd
            #A *
            #B @ A *
            #s $a A :B a
            %s :B :::fst A B s x y y

        /pair
            & $n Bool :B n tru :MkB tru

        :::snd Bool B pair
        
        same as :MkB tru
    ");

    println!("term : {}", term);
    println!("norm : {}", term::reduce(&term));
    match term::infer(&term) {
        Ok(t) => println!("type : {}", t),
        Err(e) => println!("type : {}", e)
    };

    Ok(())
}
