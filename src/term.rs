use std;
//use absal::net::*;

// Source code is Ascii-encoded.
pub type Str = [u8];
pub type Chr = u8;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Term {
    Var {idx: i32},

    // Function
    All {era: bool, nam: Vec<u8>, typ: Box<Term>, bod: Box<Term>},
    Lam {era: bool, nam: Vec<u8>, typ: Box<Term>, bod: Box<Term>},
    App {era: bool, fun: Box<Term>, arg: Box<Term>},

    // Pair (sigma)
    Sig {era: bool, nam: Vec<u8>, fst: Box<Term>, snd: Box<Term>},
    Mks {era: bool, typ: Box<Term>, fst: Box<Term>, snd: Box<Term>},
    Spt {era: bool, val: Box<Term>, ret: Box<Term>, fst: Vec<u8>, snd: Vec<u8>, bod: Box<Term>},

    // Duplication
    Dup {nam: Vec<u8>, val: Box<Term>, bod: Box<Term>},
    Bxv {val: Box<Term>}, 
    Bxt {val: Box<Term>},

    // Type
    Set
}

use self::Term::{*};

// A Scope is a vector of (name, value) assignments.
type Scope<'a> = Vec<(&'a Str, Option<Term>)>;

// Extends a scope with a (name, value) assignments.
fn extend_scope<'a,'b>(nam : &'a Str, val : Option<Term>, ctx : &'b mut Scope<'a>) -> &'b mut Scope<'a> {
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

// Parses a term, returning the remaining code and the term.
pub fn parse_term<'a>(code : &'a Str, ctx : &mut Scope<'a>, era : bool) -> (&'a Str, Term) {
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
            narrow_scope(ctx);
            let nam = nam.to_vec();
            let typ = Box::new(typ);
            let bod = Box::new(bod);
            (code, Lam{era,nam,typ,bod})
        },
        // Forall
        b'@' => {
            let (code, nam) = parse_name(&code[1..]);
            let (code, typ) = parse_term(code, ctx, false);
            let (code, bod) = parse_term(code, extend_scope(nam, None, ctx), false);
            narrow_scope(ctx);
            let nam = nam.to_vec();
            let typ = Box::new(typ);
            let bod = Box::new(bod);
            (code, All{era,nam,typ,bod})
        },
        // Sigma
        b'$' => {
            let (code, nam) = parse_name(&code[1..]);
            let (code, fst) = parse_term(code, ctx, false);
            let (code, snd) = parse_term(code, extend_scope(nam, None, ctx), false);
            narrow_scope(ctx);
            let nam = nam.to_vec();
            let fst = Box::new(fst);
            let snd = Box::new(snd);
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
            (code, Mks{era,typ,fst,snd})
        },
        // Sigma-Split
        b'%' => {
            let (code, val) = parse_term(&code[1..], ctx, false);
            let (code, ret) = parse_term(code, extend_scope(b"s", None, ctx), false);
            narrow_scope(ctx);
            let (code, fst) = parse_name(&code[1..]);
            let (code, snd) = parse_name(&code[1..]);
            let (code, bod) = parse_term(code, extend_scope(snd, None, extend_scope(fst, None, ctx)), false);
            narrow_scope(ctx);
            narrow_scope(ctx);
            let val = Box::new(val);
            let ret = Box::new(ret);
            let fst = fst.to_vec();
            let snd = snd.to_vec();
            let bod = Box::new(bod);
            (code, Spt{era,val,ret,fst,snd,bod})
        },
        // Dup
        b'=' => {
            let (code, nam) = parse_name(&code[1..]);
            let (code, val) = parse_term(code, ctx, false);
            let (code, bod) = parse_term(code, extend_scope(nam, None, ctx), false);
            narrow_scope(ctx);
            let nam = nam.to_vec();
            let val = Box::new(val);
            let bod = Box::new(bod);
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
            let mut idx : i32 = 0;
            let mut val : Option<Term> = None;
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
pub fn from_string<'a>(code : &'a Str) -> Term {
    let mut ctx = Vec::new();
    let (_code, term) = parse_term(code, &mut ctx, false);
    term
}

// Builds a var name from an index (0="a", 1="b", 26="aa"...).
pub fn var_name(idx : i32) -> Vec<Chr> {
    let mut name = Vec::new();
    let mut idx  = idx;
    if idx < 0 {
        idx = -idx;
        name.push(45);
    }
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

// Converts a λ-term back to a source-code.
pub fn to_string(term : &Term) -> Vec<Chr> {
    fn build(code : &mut Vec<u8>, term : &Term, dph : i32) {
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
            &Spt {era, ref val, ref ret, fst: ref _fst, snd: ref _snd, ref bod} => {
                if era { code.extend_from_slice(b"-"); }
                build(code, &val, dph);
                code.extend_from_slice(b" ");
                build(code, &ret, dph+1);
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

pub fn shift(proof : &mut Term, d : i32, c : i32) {
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
        &mut Sig {era: _era, nam: ref mut _nam, ref mut fst, ref mut snd} => {
            shift(fst, d, c);
            shift(snd, d, c+1);
        },
        &mut Mks {era: _era, ref mut typ, ref mut fst, ref mut snd} => {
            shift(typ, d, c);
            shift(fst, d, c);
            shift(snd, d, c);
        },
        &mut Spt {era: _era, ref mut val, ref mut ret, fst: ref _fst, snd: ref mut _snd, ref mut bod} => {
            shift(val, d, c);
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

pub fn equals(a : &Term, b : &Term) -> bool {
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
        (&Spt{era: ref _ax, val: ref ay, ret: ref az, fst: ref _aw, snd: ref _av, bod: ref au},
         &Spt{era: ref _bx, val: ref by, ret: ref bz, fst: ref _bw, snd: ref _bv, bod: ref bu})
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

pub fn subs(proof : &mut Term, value : &Term, dph : i32) {
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
        &mut Sig {era: _era, nam: ref mut _nam, ref mut fst, ref mut snd} => {
            subs(fst, value, dph);
            subs(snd, value, dph+1);
            None
        },
        &mut Mks {era: _era, ref mut typ, ref mut fst, ref mut snd} => {
            subs(typ, value, dph);
            subs(fst, value, dph);
            subs(snd, value, dph);
            None
        },
        &mut Spt {era: _era, ref mut val, ref mut ret, fst: ref _fst, snd: ref mut _snd, ref mut bod} => {
            subs(val, value, dph);
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
                shift(&mut val, dph as i32, 0);
                *proof = val
            } else if dph < idx {
                *proof = Var{idx: idx - 1}
            }
        },
        None => {}
    };
}

pub fn reduce(proof : &Term) -> Term {
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
        &Spt {era, ref val, ref ret, ref fst, ref snd, ref bod} => {
            let ret = reduce(ret);
            let val = reduce(val);
            let fst = fst.clone();
            let snd = snd.clone();
            match val {
                Mks{era: ref _mks_era, typ: ref _mks_typ, fst: ref mks_fst, snd: ref mks_snd} => {
                    let mut new_bod = *bod.clone();
                    subs(&mut new_bod, mks_snd, 0);
                    subs(&mut new_bod, mks_fst, 0);
                    reduce(&new_bod)
                },
                _ => Spt{
                    era,
                    val: Box::new(val),
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

// A Context is a vector of (name, value) assignments.
type Context<'a> = Vec<Box<Term>>;

// Extends a context.
fn extend_context<'a>(val : Box<Term>, ctx : &'a mut Context<'a>) -> &'a mut Context<'a> {
    ctx.push(val);
    for i in 0..ctx.len() {
        shift(&mut ctx[i], 1, 0);
    }
    ctx
}

// Narrows a context.
fn narrow_context<'a>(ctx : &'a mut Context<'a>) -> &'a mut Context<'a> {
    ctx.pop();
    for i in 0..ctx.len() {
        shift(&mut ctx[i], -1, 0);
    }
    ctx
}

// TODO: return Result
pub fn infer(proof : &Term) -> Result<Term, std::string::String> {
    pub fn infer<'a>(proof : &'a Term, ctx : &'a mut Context) -> Result<Term, std::string::String> {
        match proof {
            &App{era: _era, ref fun, ref arg} => {
                let fun_t = infer(fun, ctx)?;
                let arg_t = infer(arg, ctx)?;
                let arg_n = reduce(arg);
                match fun_t {
                    All{era:_era, nam: ref _nam, ref typ, ref bod} => {
                        let mut new_bod = bod.clone();
                        let a : &Term = &arg_t;
                        let b : &Term = typ;
                        if !equals(a, b) {
                            return Err(format!("Type mismatch.\nTyp: {}\nGot: {}", typ, arg_t))
                        }
                        subs(&mut new_bod, &arg_n, 0);
                        let new_bod = reduce(&new_bod);
                        Ok(new_bod)
                    },
                    _ => {
                        Err("Non-function application.".to_string())
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
            &Sig {era: _era, nam: ref _nam, fst: ref _fst, snd: ref _snd} => {
                // TODO: check fst and snd are types
                Ok(Set)
            },
            &Mks {era: _era, ref typ, ref fst, ref snd} => {
                match **typ {
                    Sig {era, nam: ref typ_nam, fst: ref typ_fst, snd: ref typ_snd} => {
                        let fst_t = reduce(typ_fst);
                        if !equals(&infer(fst, ctx)?, &fst_t) {
                            return Err("Type mismatch on first element of MkSigma.".to_string())
                        }
                        let snd_t = reduce(typ_snd);
                        let mut new_snd_t = snd_t.clone();
                        subs(&mut new_snd_t, &reduce(fst), 0);
                        if !equals(&infer(snd, ctx)?, &new_snd_t) {
                            return Err("Type mismatch on second element of MkSigma.".to_string());
                        }
                        Ok(Sig {era, nam: typ_nam.to_vec(), fst: Box::new(fst_t), snd: Box::new(snd_t)})
                    }
                    _ => Err("MkSigma type not Sigma.".to_string())
                }
            },
            &Spt {era: _era, ref val, ref ret, fst: ref _fst, snd: ref _snd, ref bod} => {
                //sigma-split : (A : Set) -> (B : A -> Set) -> (s : Sig A B) -> (P : Sig A B -> Set) -> (f : (a : A) -> (b : B a) -> P (MkSig A B a b)) -> P s
                
                // Checks if proposed return type is a type indeed
                let val_t = infer(val, ctx)?;
                let ret_v = reduce(ret);

                extend_context(Box::new(val_t.clone()), ctx);
                let ret_ret_t = infer(ret, ctx)?;
                if !equals(&ret_ret_t, &Set) {
                    return Err("Projection return type not a type.".to_string());
                }
                narrow_context(ctx);

                // Checks if projected value is a Sigma
                match val_t {
                    Sig {era, nam: ref _nam, fst: ref fst_t, snd: ref snd_t} => {
                        // Extends context with fst and snd types and gets type of body
                        let mut new_fst_t = fst_t.clone();
                        extend_context(new_fst_t, ctx);
                        let mut new_snd_t = snd_t.clone();
                        shift(&mut new_snd_t, 1, 1);
                        subs(&mut new_snd_t, &Var{idx:0}, 0);
                        extend_context(new_snd_t, ctx);
                        let bod_t = infer(bod, ctx)?;

                        // Gets the proposed return type, giving it access to the sigma's shape
                        let mut ret_t = ret_v.clone();
                        let mut mks_t = val_t.clone();
                        shift(&mut mks_t, 2, 0);
                        let mks_t = Box::new(mks_t);
                        let mks_f = Box::new(Var{idx:1});
                        let mks_s = Box::new(Var{idx:0});
                        let mks_o = &Mks{era, typ: mks_t, fst: mks_f, snd: mks_s};
                        shift(&mut ret_t, 2, 1);
                        subs(&mut ret_t, mks_o, 0);
                        let ret_t = reduce(&ret_t);

                        // Asserts that both are equal
                        if !equals(&ret_t, &bod_t) {
                            return Err(format!("Type mismatch on sigma return.\nTyp: {}\nGot: {}", ret_t, bod_t));
                        }
                        
                        // Narrows context
                        narrow_context(ctx);
                        narrow_context(ctx);

                        // Returns generalized return type
                        let mut end_t = ret_v.clone();
                        subs(&mut end_t, val, 0);
                        Ok(end_t)
                    },
                    _ => Err("Non-sigma projection.".to_string())
                }
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
                        Err("Unboxed duplication.".to_string())
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
                    _ => Err("?????????".to_string())
                }
            },
            &Set => {
                Ok(Set)
            }
        }
    }
    let mut ctx : Vec<Box<Term>> = Vec::new();
    infer(proof, &mut ctx)
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&to_string(&self)))
    }
}

//pub fn bod(proof : &Term) -> &Term {
    //match proof {
        //Lam{era:_era,nam:_nam,typ:_typ,bod} => bod,
        //_ => panic!()
    //}
//}

//pub fn to_term(proof : &Term) -> Term {
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
