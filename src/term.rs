use std;
//use absal::net::*;

// Source code is Ascii-encoded.
pub type Str = [u8];
pub type Chr = u8;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Proof {
    Var {idx: u32},

    All {era: bool, nam: Vec<u8>, typ: Box<Proof>, bod: Box<Proof>},
    Lam {era: bool, nam: Vec<u8>, typ: Box<Proof>, bod: Box<Proof>},
    App {era: bool, fun: Box<Proof>, arg: Box<Proof>},

    //data Sig : (A : Set) -> (B : A -> Set) -> Set where
    //  MkSig : (A : Set) -> (B : A -> Set) -> (a : A) -> (b : B a) -> Sig A B
    //sigma-split : (A : Set) -> (B : A -> Set) -> (s : Sig A B) -> (P : Sig A B -> Set) -> (f : (a : A) -> (b : B a) -> P (MkSig A B a b)) -> P s
    //sigma-split A B (MkSig .A .B a b) P f = f a b
    //Nat = $x Bool (case x { T => Nat ; F => Unit })
    //two = &Nat T (&Nat T (&Nat F ()))
    //one = %two Nat tag pred pred  
    Sig {era: bool, nam: Vec<u8>, fst: Box<Proof>, snd: Box<Proof>},
    Mks {era: bool, typ: Box<Proof>, fst: Box<Proof>, snd: Box<Proof>},
    Spt {era: bool, sig: Box<Proof>, ret: Box<Proof>, fst: Vec<u8>, snd: Vec<u8>, bod: Box<Proof>},

    Dup {nam: Vec<u8>, val: Box<Proof>, bod: Box<Proof>},
    Bxv {val: Box<Proof>}, 
    Bxt {val: Box<Proof>},

    Set
}

use self::Proof::{*};

// A Scope is a vector of (name, value) assignments.
type Scope<'a> = Vec<(&'a Str, Option<Proof>)>;

// Extends a scope with a (name, value) assignments.
fn extend_scope<'a,'b>(nam : &'a Str, val : Option<Proof>, ctx : &'b mut Scope<'a>) -> &'b mut Scope<'a> {
    ctx.push((nam,val));
    ctx
}

// Removes an assignment from a Scope.
fn narrow_scope<'a,'b>(ctx : &'b mut Scope<'a>) -> &'b mut Scope<'a> {
    ctx.pop();
    ctx
}

// Parses a name, returns the remaining code and the name.
fn parse_name(code : &Str) -> (&Str, &Str) {
    let mut i : usize = 0;
    while i < code.len() && !(code[i] == b' ' || code[i] == b'\n') {
        i += 1;
    }
    (&code[i..], &code[0..i])
}

pub fn parse_term<'a>(code : &'a Str, ctx : &mut Scope<'a>, era : bool) -> (&'a Str, Proof) {
    match code[0] {
        // Whitespace
        b' ' => parse_term(&code[1..], ctx, era),
        // Newline
        b'\n' => parse_term(&code[1..], ctx, era),
        // Erased term
        b'-' => {
            parse_term(&code[1..], ctx, true)
        },
        // Definition
        b'/' => {
            let (code, nam) = parse_name(&code[1..]);
            let (code, val) = parse_term(code, ctx, false);
            let (code, bod) = parse_term(code, extend_scope(nam, Some(val), ctx), false);
            narrow_scope(ctx);
            (code, bod)
        },
        // Application
        b':' => {
            let (code, fun) = parse_term(&code[1..], ctx, false);
            let (code, arg) = parse_term(code, ctx, false);
            let fun = Box::new(fun);
            let arg = Box::new(arg);
            (code, App{era,fun,arg})
        },
        // Lambda
        b'#' => {
            let (code, nam) = parse_name(&code[1..]);
            let (code, typ) = parse_term(code, ctx, false);
            let (code, bod) = parse_term(code, extend_scope(nam, None, ctx), false);
            let nam = nam.to_vec();
            let typ = Box::new(typ);
            let bod = Box::new(bod);
            narrow_scope(ctx);
            (code, Lam{era,nam,typ,bod})
        },
        // Forall
        b'@' => {
            let (code, nam) = parse_name(&code[1..]);
            let (code, typ) = parse_term(code, ctx, false);
            let (code, bod) = parse_term(code, extend_scope(nam, None, ctx), false);
            let nam = nam.to_vec();
            let typ = Box::new(typ);
            let bod = Box::new(bod);
            narrow_scope(ctx);
            (code, All{era,nam,typ,bod})
        },
        // Sigma
        b'$' => {
            let (code, nam) = parse_name(&code[1..]);
            let (code, fst) = parse_term(code, ctx, false);
            let (code, snd) = parse_term(code, extend_scope(nam, None, ctx), false);
            let nam = nam.to_vec();
            let fst = Box::new(fst);
            let snd = Box::new(snd);
            narrow_scope(ctx);
            (code, Sig{era,nam,fst,snd})
        },
        // MkSigma
        b'&' => {
            let (code, typ) = parse_term(&code[1..], ctx, false);
            let (code, fst) = parse_term(code, ctx, false);
            let (code, snd) = parse_term(code, ctx, false);
            let typ = Box::new(typ);
            let fst = Box::new(fst);
            let snd = Box::new(snd);
            narrow_scope(ctx);
            (code, Mks{era,typ,fst,snd})
        },
        // Projection
        b'%' => {
            let (code, sig) = parse_term(&code[1..], ctx, false);
            let (code, ret) = parse_term(code, extend_scope(b"s", None, ctx), false);
            let (code, fst) = parse_name(code);
            let (code, snd) = parse_name(code);
            let (code, bod) = parse_term(code, extend_scope(fst, None, extend_scope(snd, None, ctx)), false);
            let sig = Box::new(sig);
            let ret = Box::new(ret);
            let fst = fst.to_vec();
            let snd = snd.to_vec();
            let bod = Box::new(bod);
            narrow_scope(ctx);
            narrow_scope(ctx);
            (code, Spt{era,sig,ret,fst,snd,bod})
        },
        // Dup
        b'=' => {
            let (code, nam) = parse_name(&code[1..]);
            let (code, val) = parse_term(code, ctx, false);
            let (code, bod) = parse_term(code, extend_scope(nam, None, ctx), false);
            let nam = nam.to_vec();
            let val = Box::new(val);
            let bod = Box::new(bod);
            narrow_scope(ctx);
            (code, Dup{nam,val,bod})
        },
        // Bxv
        b'|' => {
            let (code, val) = parse_term(&code[1..], ctx, false);
            let val = Box::new(val);
            (code, Bxv{val})
        },
        // Bxt
        b'!' => {
            let (code, val) = parse_term(&code[1..], ctx, false);
            let val = Box::new(val);
            (code, Bxt{val})
        },
        // Set
        b'*' => {
            (&code[1..], Set)
        },
        // Variable
        _ => {
            let (code, nam) = parse_name(code);
            let mut idx : u32 = 0;
            let mut val : Option<Proof> = None;
            for i in (0..ctx.len()).rev() {
                if ctx[i].0 == nam {
                    val = ctx[i].1.clone();
                    break;
                }
                idx = idx + (match &ctx[i].1 { &Some(ref _t) => 0, &None => 1});
            }
            (code, match val { Some(term) => term, None => Var{idx} })
        }
    }
}

// Converts a source-code to a λ-term.
pub fn from_string<'a>(code : &'a Str) -> Proof {
    let mut ctx = Vec::new();
    let (_code, term) = parse_term(code, &mut ctx, false);
    term
}

// Builds a var name from an index (0="a", 1="b", 26="aa"...).
pub fn var_name(idx : u32) -> Vec<Chr> {
    let mut name = Vec::new();
    let mut idx  = idx;
    if idx == 0 {
        name.push(63);
    }
    while idx > 0 {
        idx = idx - 1;
        name.push((97 + idx % 26) as u8);
        idx = idx / 26;
    }
    return name;
}

//// Converts a λ-term back to a source-code.
pub fn to_string(term : &Proof) -> Vec<Chr> {
    fn build(code : &mut Vec<u8>, term : &Proof, dph : u32) {
        match term {
            &App{era, ref fun, ref arg} => {
                if era { code.extend_from_slice(b"-"); }
                code.extend_from_slice(b":");
                build(code, &fun, dph);
                code.extend_from_slice(b" ");
                build(code, &arg, dph);
            },
            &Lam{era, nam: ref _nam, ref typ, ref bod} => {
                if era { code.extend_from_slice(b"-"); }
                code.extend_from_slice(b"#");
                code.append(&mut var_name(dph + 1));
                code.extend_from_slice(b" ");
                build(code, &typ, dph);
                code.extend_from_slice(b" ");
                build(code, &bod, dph + 1);
            },
            &All{era, nam: ref _nam, ref typ, ref bod} => {
                if era { code.extend_from_slice(b"-"); }
                code.extend_from_slice(b"@");
                code.append(&mut var_name(dph + 1));
                code.extend_from_slice(b" ");
                build(code, &typ, dph);
                code.extend_from_slice(b" ");
                build(code, &bod, dph + 1);
            },
            &Sig {era, nam: ref _nam, ref fst, ref snd} => {
                if era { code.extend_from_slice(b"-"); }
                code.extend_from_slice(b"$");
                code.append(&mut var_name(dph + 1));
                code.extend_from_slice(b" ");
                build(code, &fst, dph);
                code.extend_from_slice(b" ");
                build(code, &snd, dph+1);
            },
            &Mks {era, ref typ, ref fst, ref snd} => {
                if era { code.extend_from_slice(b"-"); }
                code.extend_from_slice(b"&");
                build(code, &typ, dph);
                code.extend_from_slice(b" ");
                build(code, &fst, dph);
                code.extend_from_slice(b" ");
                build(code, &snd, dph);
            },
            &Spt {era, ref sig, ref ret, fst: ref _fst, snd: ref _snd, ref bod} => {
                if era { code.extend_from_slice(b"-"); }
                build(code, &sig, dph);
                code.extend_from_slice(b" ");
                build(code, &ret, dph);
                code.extend_from_slice(b" ");
                code.append(&mut var_name(dph + 1));
                code.extend_from_slice(b" ");
                code.append(&mut var_name(dph + 2));
                code.extend_from_slice(b" ");
                build(code, &bod, dph+2);
            },
            &Var{idx} => {
                code.append(&mut var_name(dph - idx));
            },
            &Dup{nam: ref _nam, ref val, ref bod} => {
                code.extend_from_slice(b"=");
                code.append(&mut var_name(dph + 1));
                code.extend_from_slice(b" ");
                build(code, &val, dph);
                code.extend_from_slice(b" ");
                build(code, &bod, dph + 1);
            },
            &Bxv{ref val} => {
                code.extend_from_slice(b"|");
                build(code, &val, dph);
            },
            &Bxt{ref val} => {
                code.extend_from_slice(b"!");
                build(code, &val, dph);
            },
            &Set => {
                code.extend_from_slice(b"*");
            }
        }
    }
    let mut code = Vec::new();
    build(&mut code, term, 0);
    return code;
}

pub fn shift(proof : &mut Proof, d : u32, c : u32) {
    match proof {
        &mut App{era: _era, ref mut fun, ref mut arg} => {
            shift(fun, d, c);
            shift(arg, d, c);
        },
        &mut Lam{era: _era, nam: ref mut _nam, ref mut typ, ref mut bod} => {
            shift(typ, d, c);
            shift(bod, d, c+1);
        },
        &mut All{era: _era, nam: ref mut _nam, ref mut typ, ref mut bod} => {
            shift(typ, d, c);
            shift(bod, d, c+1);
        },
        &mut Var{ref mut idx} => {
            *idx = if *idx < c { *idx } else { *idx + d };
        },
        &mut Sig {era, nam: ref mut _nam, ref mut fst, ref mut snd} => {
            shift(fst, d, c);
            shift(snd, d, c+1);
        },
        &mut Mks {era, ref mut typ, ref mut fst, ref mut snd} => {
            shift(typ, d, c);
            shift(fst, d, c);
            shift(snd, d, c);
        },
        &mut Spt {era, ref mut sig, ref mut ret, fst: ref _fst, snd: ref mut _snd, ref mut bod} => {
            shift(sig, d, c);
            shift(ret, d, c+1);
            shift(bod, d, c+2);
        },
        &mut Dup{nam: ref mut _nam, ref mut val, ref mut bod} => {
            shift(val, d, c);
            shift(bod, d, c+1);
        },
        &mut Bxv{ref mut val} => {
            shift(val, d, c);
        },
        &mut Bxt{ref mut val} => {
            shift(val, d, c);
        },
        &mut Set => {}
    }
}

pub fn equals(a : &Proof, b : &Proof) -> bool {
    match (a,b) {
        (&App{era: ref _ax, fun: ref ay, arg: ref az},
         &App{era: ref _bx, fun: ref by, arg: ref bz})
         => equals(ay,by) && equals(az,bz),
        (&Lam{era: ref _ax, nam: ref _ay, typ: ref az, bod: ref aw},
         &Lam{era: ref _bx, nam: ref _by, typ: ref bz, bod: ref bw})
         => equals(az,bz) && equals(aw,bw),
        (&All{era: ref _ax, nam: ref _ay, typ: ref az, bod: ref aw},
         &All{era: ref _bx, nam: ref _by, typ: ref bz, bod: ref bw})
         => equals(az,bz) && equals(aw,bw),
        (&Var{idx: ref ax},
         &Var{idx: ref bx})
         => ax == bx,
        (&Sig{era: ref _ax, nam: ref _ay, fst: ref az, snd: ref aw},
         &Sig{era: ref _bx, nam: ref _by, fst: ref bz, snd: ref bw})
         => equals(az,bz) && equals(aw,bw),
        (&Mks{era: ref _ax, typ: ref az, fst: ref aw, snd: ref av},
         &Mks{era: ref _bx, typ: ref bz, fst: ref bw, snd: ref bv})
         => equals(az,bz) && equals(aw,bw) && equals(av,bv),
        (&Spt{era: ref _ax, sig: ref ay, ret: ref az, fst: ref _aw, snd: ref _av, bod: ref au},
         &Spt{era: ref _bx, sig: ref by, ret: ref bz, fst: ref _bw, snd: ref _bv, bod: ref bu})
         => equals(ay,by) && equals(az,bz) && equals(au,bu),
        (&Dup{nam: ref _ax, val: ref ay, bod: ref az},
         &Dup{nam: ref _bx, val: ref by, bod: ref bz})
         => equals(ay, by) && equals(az, bz),
        (&Bxv{val: ref ax}, &Bxv{val: ref bx})
         => equals(ax, bx),
        (&Bxt{val: ref ax}, &Bxt{val: ref bx})
         => equals(ax, bx),
        (Set, Set)
         => true,
        _ => false
    }
}

pub fn subs(proof : &mut Proof, value : &Proof, dph : u32) {
    let var_idx = match proof {
        &mut App{era: _era, ref mut fun, ref mut arg} => {
            subs(fun, value, dph);
            subs(arg, value, dph);
            None
        },
        &mut Lam{era: _era, nam: ref mut _nam, ref mut typ, ref mut bod} => {
            subs(typ, value, dph);
            subs(bod, value, dph+1);
            None
        },
        &mut All{era: _era, nam: ref mut _nam, ref mut typ, ref mut bod} => {
            subs(typ, value, dph);
            subs(bod, value, dph+1);
            None
        },
        &mut Var{idx} => {
            Some(idx)
        },
        &mut Sig {era, nam: ref mut _nam, ref mut fst, ref mut snd} => {
            subs(fst, value, dph);
            subs(snd, value, dph+1);
            None
        },
        &mut Mks {era, ref mut typ, ref mut fst, ref mut snd} => {
            subs(typ, value, dph);
            subs(fst, value, dph);
            subs(snd, value, dph);
            None
        },
        &mut Spt {era, ref mut sig, ref mut ret, fst: ref _fst, snd: ref mut _snd, ref mut bod} => {
            subs(sig, value, dph);
            subs(ret, value, dph+1);
            subs(bod, value, dph+2);
            None
        },
        &mut Dup{nam: ref mut _nam, ref mut val, ref mut bod} => {
            subs(val, value, dph);
            subs(bod, value, dph+1);
            None
        },
        &mut Bxv{ref mut val} => {
            subs(val, value, dph);
            None
        },
        &mut Bxt{ref mut val} => {
            subs(val, value, dph);
            None
        },
        &mut Set => {
            None
        }
    };
    // Because couldn't modify Var inside its own case
    match var_idx {
        Some(idx) => {
            if dph == idx {
                let mut val = value.clone();
                shift(&mut val, dph, 0);
                *proof = val
            } else if dph < idx {
                *proof = Var{idx: idx - 1}
            }
        },
        None => {}
    };
}

pub fn reduce(proof : &Proof) -> Proof {
    match proof {
        &App{era, ref fun, ref arg} => {
            let fun = reduce(fun);
            match fun {
                Lam{era:_era, nam:_nam, typ:_typ, bod} => {
                    let mut new_bod = *bod.clone();
                    subs(&mut new_bod, arg, 0);
                    reduce(&new_bod)
                },
                Dup{nam: vnam, val: vval, bod: vbod} => {
                    let new_bod = Box::new(reduce(&App{era, fun: vbod, arg: Box::new(*arg.clone())})); 
                    Dup{nam: vnam, val: vval, bod: new_bod}
                },
                _ => App{era, fun: Box::new(fun), arg: Box::new(reduce(&arg))}
            }
        },
        &Lam{era, ref nam, ref typ, ref bod} => {
            let typ = Box::new(reduce(typ));
            let bod = Box::new(reduce(bod));
            Lam{era,nam:nam.to_vec(),typ,bod}
        },
        &All{era, ref nam, ref typ, ref bod} => {
            let typ = Box::new(reduce(typ));
            let bod = Box::new(reduce(bod));
            All{era,nam:nam.to_vec(),typ,bod}
        },
        &Var{idx} => {
            Var{idx}
        },
        &Sig {era, ref nam, ref fst, ref snd} => {
            let nam = nam.clone();
            let fst = Box::new(reduce(fst));
            let snd = Box::new(reduce(snd));
            Sig{era, nam, fst, snd}
        },
        &Mks {era, ref typ, ref fst, ref snd} => {
            let typ = Box::new(reduce(typ));
            let fst = Box::new(reduce(fst));
            let snd = Box::new(reduce(snd));
            Mks{era, typ, fst, snd}
        },
        &Spt {era, ref sig, ref ret, ref fst, ref snd, ref bod} => {
            let sig = reduce(sig);
            let ret = reduce(ret);
            let fst = fst.clone();
            let snd = snd.clone();
            match sig {
                Mks{era: ref _mks_era, typ: ref _mks_typ, fst: ref mks_fst, snd: ref mks_snd} => {
                    let mut new_bod = *bod.clone();
                    subs(&mut new_bod, mks_fst, 0);
                    subs(&mut new_bod, mks_snd, 1);
                    reduce(&new_bod)
                },
                _ => Spt{
                    era,
                    sig: Box::new(sig),
                    ret: Box::new(ret),
                    fst: fst,
                    snd: snd,
                    bod: Box::new(reduce(&bod))
                }
            }
        },
        &Dup{ref nam, ref val, ref bod} => {
            let val = reduce(val);
            match val {
                Bxv{ref val} => {
                    let mut new_bod = *bod.clone();
                    subs(&mut new_bod, val, 0);
                    reduce(&new_bod)
                },
                Dup{nam: vnam, val: vval, bod: vbod} => {
                    let new_bod = Box::new(reduce(&Dup{nam:nam.to_vec(), val: vbod, bod: Box::new(*bod.clone())})); 
                    Dup{nam: vnam, val: vval, bod: new_bod}
                },
                _ => {
                    let val = Box::new(val);
                    let bod = Box::new(reduce(bod));
                    Dup{nam:nam.to_vec(),val,bod}
                }
            }
        },
        &Bxv{ref val} => {
            let val = Box::new(reduce(val));
            Bxv{val}
        },
        &Bxt{ref val} => {
            let val = Box::new(reduce(val));
            Bxt{val}
        },
        Set => {
            Set
        }
    }
}

// TODO: return Result
//pub fn is_stratified(proof : &Proof) -> bool {
    //pub fn check<'a>(proof : &'a Proof, lvl : u32, ctx : &'a mut Vec<(Vec<u8>,VarType,bool,u32)>) { // name, is_exponential, was_used, level
        //match proof {
            //&App{era: _era, ref fun, ref arg} => {
                //check(fun, lvl, ctx);
                //check(arg, lvl, ctx);
            //},
            //&Lam{era, ref nam, ref typ, ref bod} => {
                //check(typ, lvl, ctx);
                //ctx.push((nam.to_vec(), Affine, false, lvl));
                //check(bod, lvl, ctx);
                //ctx.pop();
            //},
            //&All{era, ref nam, ref typ, ref bod} => {
                //check(typ, lvl, ctx);
                //ctx.push((nam.to_vec(), Affine, false, lvl));
                //check(bod, lvl, ctx);
                //ctx.pop();
            //},
            //&Var{idx} => {
                //let pos = ctx.len() - idx as usize - 1;
                //let var_nam = ctx[pos].0.clone();
                //let var_typ = ctx[pos].1.clone();
                //let var_use = ctx[pos].2;
                //let var_lvl = ctx[pos].3;
                //match var_typ {
                    //Affine => {
                        //if var_use {
                            //panic!("Affine variable '{}' used more than once.", std::str::from_utf8(&var_nam).unwrap());
                        //}
                        //if lvl > var_lvl {
                            //panic!("Affine variable '{}' lost its scope.", std::str::from_utf8(&var_nam).unwrap());
                        //}
                    //},
                    //Exponential => {
                        //if lvl - var_lvl != 1 {
                            //panic!("Exponential variables should have surrounding box, but '{}' has {}.", std::str::from_utf8(&var_nam).unwrap(), lvl - var_lvl);
                        //}
                    //}
                //}
                //ctx[pos].2 = true;
            //},
            //&Dup{ref nam, ref val, ref bod} => {
                //check(val, lvl, ctx);
                //ctx.push((nam.to_vec(), Exponential, false, lvl));
                //check(bod, lvl, ctx);
                //ctx.pop();
            //},
            //&Fix{ref nam,ref bod} => {
                //ctx.push((nam.to_vec(), Exponential, false, lvl));
                //check(bod, lvl, ctx);
            //},
            //&Bxv{ref val} => {
                //check(val, lvl + 1, ctx);
            //},
            //&Bxt{ref val} => {
                //check(val, lvl, ctx);
            //},
            //&Set => {
            //}
        //}
    //};
    //let mut ctx : Vec<(Vec<u8>,VarType,bool,u32)> = Vec::new();
    //check(proof, 0, &mut ctx);
    //true
//}

// A Context is a vector of (name, value) assignments.
type Context<'a> = Vec<Box<Proof>>;

// Extends a context.
fn extend_context<'a>(val : Box<Proof>, ctx : &'a mut Context<'a>) -> &'a mut Context<'a> {
    ctx.push(val);
    for i in 0..ctx.len() {
        shift(&mut ctx[i], 1, 0);
    }
    ctx
}

// Narrows a context.
fn narrow_context<'a>(ctx : &'a mut Context<'a>) -> &'a mut Context<'a> {
    ctx.pop();
    ctx
}

// TODO: return Result
pub fn infer(proof : &Proof) -> Result<Proof, &'static str> {
    pub fn infer<'a>(proof : &'a Proof, ctx : &'a mut Context) -> Result<Proof, &'static str> {
        match proof {
            &App{era: _era, ref fun, ref arg} => {
                let fun_t = infer(fun, ctx)?;
                let arg_t = infer(arg, ctx)?;
                let arg_n = reduce(arg);
                match fun_t {
                    All{era:_era, nam: ref _nam, ref typ, ref bod} => {
                        let mut new_bod = bod.clone();
                        let a : &Proof = &arg_t;
                        let b : &Proof = typ;
                        if !equals(a, b) {
                            return Err("Type mismatch.")
                        }
                        subs(&mut new_bod, &arg_n, 0);
                        Ok(*new_bod)
                    },
                    _ => {
                        println!("??? {} | {}", fun_t, arg_t);
                        Err("Non-function application.")
                    }
                }
            },
            &Lam{era, ref nam, ref typ, ref bod} => {
                let typ_n = Box::new(reduce(typ));
                extend_context(typ_n.clone(), ctx);
                let bod_t = Box::new(infer(bod, ctx)?);
                narrow_context(ctx);
                Ok(All{era, nam: nam.to_vec(), typ: typ_n, bod: bod_t})
            },
            &All{era: _era, nam: ref _nam, ref typ, bod: ref _bod} => {
                let typ_n = Box::new(reduce(typ));
                extend_context(typ_n, ctx);
                // TODO: valid forall check
                narrow_context(ctx);
                Ok(Set)
            },
            &Var{idx} => {
                Ok(*ctx[ctx.len() - (idx as usize) - 1].clone())
            },
            &Sig {era: _era, nam: ref _nam, ref fst, ref snd} => {
                // TODO: check fst and snd are types
                Ok(Set)
            },
            &Mks {era: _era, ref typ, ref fst, ref snd} => {
                match **typ {
                    Sig {era, nam: ref typ_nam, fst: ref typ_fst, snd: ref typ_snd} => {
                        let fst_t = reduce(typ_fst);
                        if !equals(&infer(fst, ctx)?, &fst_t) {
                            return Err("Type mismatch on first element of MkSigma.")
                        }
                        let snd_t = reduce(typ_snd);
                        let mut new_snd_t = snd_t.clone();
                        subs(&mut new_snd_t, &reduce(fst), 0);
                        if !equals(&infer(snd, ctx)?, &new_snd_t) {
                            return Err("Type mismatch on second element of MkSigma.");
                        }
                        Ok(Sig {era, nam: typ_nam.to_vec(), fst: Box::new(fst_t), snd: Box::new(snd_t)})
                    }
                    _ => Err("MkSigma type not Sigma.")
                }
            },
            &Spt {era: _era, sig: ref _sig, ret: ref _ret, fst: ref _fst, snd: ref _snd, bod: ref _bod} => {
                panic!("quer demais bb");
                //let sig_t = infer(sig, ctx)?;
                //match sig_t {
                    //Sig {era: sig_era, nam: ref sig_nam, fst: ref sig_fst, snd: ref sig_snd} => {
                        //let mut new_bod = bod.clone();


                    //All{era:_era, nam: ref _nam, ref typ, ref bod} => {
                        //let mut new_bod = bod.clone();
                        //let a : &Proof = &arg_t;
                        //let b : &Proof = typ;
                        //if !equals(a, b) {
                            //return Err("Type mismatch.")
                        //}
                        //subs(&mut new_bod, &arg_n, 0);
                        
                    //},
                    //_ => Err("Non sigma projection.")
                //}
                //let arg_t = infer(arg, ctx)?;
                //let arg_n = reduce(arg);
                //let fst = fst.clone();
                //let snd = snd.clone();
                //let sig = reduce(sig);
                //match sig {
                    //Mks{era: ref mks_era, typ: ref mks_typ, fst: ref mks_fst, snd: ref mks_snd} => {
                        //let mut new_bod = *bod.clone();
                        //subs(&mut new_bod, mks_fst, 0);
                        //subs(&mut new_bod, mks_snd, 1);
                        //reduce(&new_bod)
                    //},
                    //_ => Mks{era, fun, snd, sig: Box::new(sig), bod: Box::new(reduce(&bod))}
                //}
            },
            &Dup{nam: ref _nam, ref val, ref bod} => {
                let val_t = infer(val, ctx)?;
                let val_n = reduce(val);
                match val_t {
                    Bxt{val: val_t} => {
                        extend_context(val_t, ctx);
                        let mut bod_t = infer(bod, ctx)?;
                        narrow_context(ctx);
                        subs(&mut bod_t, &val_n, 0);
                        Ok(bod_t)
                    },
                    _ => {
                        Err("Unboxed duplication.")
                    }
                }
            },
            &Bxv{ref val} => {
                let val_t = infer(val, ctx)?;
                Ok(Bxt{val: Box::new(val_t)})
            },
            &Bxt{ref val} => {
                let val_t = infer(val, ctx)?;
                match val_t {
                    Set => Ok(Set),
                    _ => Err("?????????")
                }
            },
            &Set => {
                Ok(Set)
            }
        }
    }
    let mut ctx : Vec<Box<Proof>> = Vec::new();
    infer(proof, &mut ctx)
}

impl std::fmt::Display for Proof {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&to_string(&self)))
    }
}

//pub fn bod(proof : &Proof) -> &Proof {
    //match proof {
        //Lam{era:_era,nam:_nam,typ:_typ,bod} => bod,
        //_ => panic!()
    //}
//}

//pub fn to_term(proof : &Proof) -> Term {
    //match proof {
        //App {era, ref fun, ref arg} => {
            //Term::App{
                //fun: Box::new(to_term(fun)),
                //arg: Box::new(to_term(arg))
            //}
        //},
        //Lam {era, ref nam, ref typ, ref bod} => {
            //Term::Lam{
                //bod: Box::new(to_term(bod))
            //}
        //},
        //All {era, ref nam, ref typ, ref bod} => {
            //panic!("ue")
        //},
        //Var {idx} => {
            //Term::Var{idx: *idx}
        //},
        //Dup {ref nam, ref val, ref bod} => {
            //let mut new_bod = *bod.clone();
            //subs(&mut new_bod, val);
            //to_term(&new_bod)
        //},
        //Fix {ref nam, ref bod} => panic!(),
        //Bxv {ref val} => panic!(),
        //Bxt {ref val} => panic!(),
        //Set => panic!()
    //}
//}
