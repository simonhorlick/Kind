//fn main() {
    //let a : u32 = 10;
    //let ma = HashMap::singleton("a", 1);
    //let mb = ma.update("b", 2);
    //let mc = mb.update("c", 3);
    //let md = mc.update("d", 4);
    //let me = md.update("e", 5);
    //println!("{:?}", mb)
//}

pub mod term;
use term::*;

pub fn get(name : &[u8], env : &Env) -> Term {
    env .get(&name.to_vec())
        .expect("Attempted to run unexisting program.")
        .clone()
}

pub fn nf(name : &[u8], env : &Env) -> Term {
    let mut val = get(name, env);
    term::reduce(&mut val, &env);
    val
}

pub fn ty(name : &[u8], env : &Env) -> Result<Term, std::string::String> {
    let val = get(name, env);
    term::infer(&val, &env)
}

//pub fn typ(name : &Vec<u8>, env : &Env) -> Option<Term> {
    //let mut val = env.get(name)?.clone();
    //term::infer(&mut val, &env);
    //val
//}

fn main() -> Result<(), std::string::String> {
    let env : Env = build(b"
        =id   #P * #x P x
        =zer  #P * #S @ P P #Z P Z
        =Bits @P * @O @ * P #I @ * P #E P P
        =O    #xs * #P * #O @ * P #I @ * P #E P :O xs
        =I    #xs * #P * #O @ * P #I @ * P #E P :I xs
        =E    #P * #O @ * P #I @ * P #E P E
        =list :I :O main
        =tail #xs * ::::xs * #xs * xs #xs * xs E
        =fst  #xs * ::::xs *
            #xs * #a * #b * a
            #xs * #a * #b * b
            #x * x
        =main :tail :tail :tail list
    ");

    println!("{}", nf(b"main", &env));


    //println!("{}", ty(b"zer", &env)?);



    //for (nam,ter) in prog.clone() {
        //println!("{} {}", String::from_utf8_lossy(&nam), ter);
    //}

    //let main : Term = prog.get(&b"zer".to_vec())?;
    //println!("{}", main);






    //let mut defs : HashMap<Vec<u8>, Term> = HashMap::new();
    //defs.insert(b"id".to_vec(), term::from_string(b"#P * #x P x")); 
    //println!("{:?}", prog);
    //let defs = 
        //"id", "#P * #x P x"
    //};
    //let main = "#t * id";

    ////let mut term = term::from_string(b"
        ////=x #a * a
        ////x


        /////Y
            ////#f * :#x * :f :x x #x * :f :x x

        /////ones
            ////:Y #R * #O * #I * #E * :O R

        /////fst_bit
            ////#bs *
            ////:::bs
                ////#bs * #a * #b * a
                ////#bs * #a * #b * b
                ////#a * a

        ////Y
        
        /////Bool
            ////@P *
            ////@t P
            ////@f P
            ////P

        /////false
            ////#P *
            ////#T P
            ////#F P
            ////F

        /////true
            ////#P *
            ////#T P
            ////#F P
            ////T

        /////not
            ////#a Bool
            ////#P *
            ////#T P
            ////#F P
            ////:::a P F T

        /////Pair
            ////#A *
            ////#B @ A *
            ////@P *
            ////@t @a A @b :B a P
            ////P

        /////pair
            ////#A *
            ////#B @ A *
            ////#a A 
            ////#b :B a
            ////#P *
            ////#t @a A @b :B a P
            ////::t a b

        /////fst
            ////#A *
            ////#B @ A *
            ////#t ::Pair A B
            ////::t A #a A #b :B a a

        /////snd
            ////#A *
            ////#B @ A *
            ////#t ::Pair A B
            ////::t :B a #a A #b :B a b

        ////snd

        ////@a *
        ////@b @c b *
        ////@c  @d *
            ////@e @f b @g :c f d
            ////d
        ////@d @e a
           ////@f :b e
           ////a
        ////a

        ////Bool
        ////:not :not :not true

        ////:   #s * #z * :s :s :s :s :s z
            ////#s * #z * :s :s :s :s :s z
            

        /////B
            ////#b Bool
            ////@P @b Bool *
            ////@M @b Bool :P b
            ////:P b

        /////not
            ////#a Bool
            ////:::a Bool fal tru

        /////Nat
            ////@P *
            ////@s @ P P
            ////@z P
            ////P

        /////suc
            ////#n Nat
            ////#P *
            ////#s @ P P
            ////#z P
            ////:::n P s z

        /////zer
            ////#P *
            ////#s @ P P
            ////#z P
            ////z

        /////MkB
            ////#b Bool
            ////#P @b Bool *
            ////#M @b Bool :P b
            ////:M b

        /////fst
            ////#A * 
            ////#B @ A *
            ////#s $a A :B a
            ////%s A x y x

        /////snd
            ////#A *
            ////#B @ A *
            ////#s $a A :B a
            ////%s :B :::fst A B s x y y

        /////pair
            ////& $n Bool :B n tru :MkB tru

        /////S #n * #s * #z * :s n
        /////Z      #s * #z * z
        /////I ~I #n * ::n #n * :S :I n Z
        /////A ~A #n * ::n
            ////#n * #m * ::A :S n m
            ////#m * m
        
        ////:I Z

    //println!("term : {}", term);
    //term::reduce(&mut term, &defs);
    ////match term::infer(&term) {
        ////Ok(typ) => println!("type : {}", typ),
        ////Err(err) => println!("type : {}", err)
    ////};
    //println!("norm : {}", term);

    Ok(())
}
