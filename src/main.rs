pub mod term;

fn main() -> Result<(), &'static str> {
    let mut term = term::from_string(b"
        /Bool
            @P *
            @t P
            @f P
            P

        /false
            #P *
            #T P
            #F P
            F

        /true
            #P *
            #T P
            #F P
            T

        /not
            #a Bool
            #P *
            #T P
            #F P
            :::a P F T

        /Pair
            #A *
            #B @ A *
            @P *
            @t @a A @b :B a P
            P

        /pair
            #A *
            #B @ A *
            #a A 
            #b :B a
            #P *
            #t @a A @b :B a P
            ::t a b

        /fst
            #A *
            #B @ A *
            #t ::Pair A B
            ::t A #a A #b :B a a

        #F @ * F :F *

        /snd
            #A *
            #B @ A *
            #t ::Pair A B
            ::t :B a #a A #b :B a b

        snd

        @a *
        @b @c b *
        @c  @d *
            @e @f b @g :c f d
            d
        @d @e a
           @f :b e
           a
        a

        Bool
        :not :not :not true

        :   #s * #z * :s :s :s :s :s z
            #s * #z * :s :s :s :s :s z
            

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

        /S #n * #s * #z * :s n
        /Z      #s * #z * z
        /I ~I #n * ::n #n * :S :I n Z
        /A ~A #n * ::n
            #n * #m * ::A :S n m
            #m * m
        
        :I Z

    ");
    //same as :MkB tru

    println!("term : {}", term);
    term::reduce(&mut term);
    match term::infer(&term) {
        Ok(typ) => println!("type : {}", typ),
        Err(err) => println!("type : {}", err)
    };
    println!("norm : {}", term);

    Ok(())
}
