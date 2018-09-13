extern crate absal;

pub mod term;


fn main() -> Result<(), &'static str> {
    let term = term::from_string(b"
        /bool
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

        /not
            #a bool
            :::a bool fal tru

        /nat
            @P *
            @s @ P P
            @z P
            P

        /suc
            #n nat
            #P *
            #s @ P P
            #z P
            :::n P s z

        /zer
            #P *
            #s @ P P
            #z P
            z

        :suc :suc zer
    ");
    println!("term : {}", term);
    println!("norm : {}", term::reduce(&term));
    println!("type : {}", term::infer(&term)?);
    Ok(())
    //let inp     = absal::term::from_string(b"@ #x #y x #k k");
    //let mut net = absal::term::to_net(&inp);
    //let stt     = absal::net::reduce(&mut net);
    //let out     = absal::term::from_net(&net);
    //println!("nn  : {}", inp);
    //println!("out : {}", out);
    //println!("stt : {:?}", stt);
}
