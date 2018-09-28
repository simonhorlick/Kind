use std;
use std::collections::HashMap;
//use absal::net::*;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Term {
    All {nam: Vec<u8>, typ: Box<Term>, bod: Box<Term>},
    Lam {nam: Vec<u8>, typ: Box<Term>, bod: Box<Term>},
    Var {idx: i32},
    App {fun: Box<Term>, arg: Box<Term>},
    Ref {nam: Vec<u8>},
    //Let {nam: Vec<u8>, val: Box<Term>, bod: Box<Term>},
    //Let {nam: Vec<u8>, val: Box<Term>, bod: Box<Term>},
    //Bxv {val: Box<Term>}, 
    //Bxt {val: Box<Term>},

    // Type
    Set
}

pub type Env = HashMap<Vec<u8>, Term>;

use self::Term::{*};

// A Scope is a vector of (name, value) assignments.
type Scope<'a> = Vec<(&'a [u8], Option<Term>)>;

// Extends a scope with a (name, value) assignments.
fn extend_scope<'a,'b>(nam : &'a [u8], val : Option<Term>, ctx : &'b mut Scope<'a>) -> &'b mut Scope<'a> {
    ctx.push((nam,val));
    ctx
}

// Removes an assignment from a Scope.
fn narrow_scope<'a,'b>(ctx : &'b mut Scope<'a>) -> &'b mut Scope<'a> {
    ctx.pop();
    ctx
}

// Parses a name, returns the remaining code and the name.
fn parse_name(code : &[u8]) -> (&[u8], &[u8]) {
    let mut i : usize = 0;
    while i < code.len() && !(code[i] == b' ' || code[i] == b'\n') {
        i += 1;
    }
    (&code[i..], &code[0..i])
}

// Parses a term, returning the remaining code and the term.
pub fn parse_term<'a>(code : &'a [u8], ctx : &mut Scope<'a>) -> (&'a [u8], Term) {
    match code[0] {
        // Whitespace
        b' ' => parse_term(&code[1..], ctx),
        // Newline
        b'\n' => parse_term(&code[1..], ctx),
        // Definition
        b'/' => {
            let (code, nam) = parse_name(&code[1..]);
            let (code, val) = parse_term(code, ctx);
            let (code, bod) = parse_term(code, extend_scope(nam, Some(val), ctx));
            narrow_scope(ctx);
            (code, bod)
        },
        // Application
        b':' => {
            let (code, fun) = parse_term(&code[1..], ctx);
            let (code, arg) = parse_term(code, ctx);
            let fun = Box::new(fun);
            let arg = Box::new(arg);
            (code, App{fun,arg})
        },
        // Lambda
        b'#' => {
            let (code, nam) = parse_name(&code[1..]);
            extend_scope(nam, None, ctx);
            let (code, typ) = parse_term(code, ctx);
            let (code, bod) = parse_term(code, ctx);
            narrow_scope(ctx);
            let nam = nam.to_vec();
            let typ = Box::new(typ);
            let bod = Box::new(bod);
            (code, Lam{nam,typ,bod})
        },
        // Forall
        b'@' => {
            let (code, nam) = parse_name(&code[1..]);
            extend_scope(nam, None, ctx);
            let (code, typ) = parse_term(code, ctx);
            let (code, bod) = parse_term(code, ctx);
            narrow_scope(ctx);
            let nam = nam.to_vec();
            let typ = Box::new(typ);
            let bod = Box::new(bod);
            (code, All{nam,typ,bod})
        },
        // Let
        //b'=' => {
            //let (code, nam) = parse_name(&code[1..]);
            //let (code, val) = parse_term(code, ctx);
            //let (code, bod) = parse_term(code, ctx);
            //let nam = nam.to_vec();
            //let val = Box::new(val);
            //let bod = Box::new(bod);
            //(code, Let{nam,val,bod})
        //},
        // Bxv
        //b'|' => {
            //let (code, val) = parse_term(&code[1..], ctx, false);
            //let val = Box::new(val);
            //(code, Bxv{val})
        //},
        // Bxt
        //b'!' => {
            //let (code, val) = parse_term(&code[1..], ctx, false);
            //let val = Box::new(val);
            //(code, Bxt{val})
        //},
        // Set
        b'*' => {
            (&code[1..], Set)
        },
        // Variable
        _ => {
            let (code, nam) = parse_name(code);
            let mut idx : i32 = 0;
            let mut fnd : bool = false;
            let mut val : Option<Term> = None;
            for i in (0..ctx.len()).rev() {
                if ctx[i].0 == nam {
                    val = ctx[i].1.clone();
                    fnd = true;
                    break;
                }
                idx = idx + (match &ctx[i].1 { &Some(ref _t) => 0, &None => 1 });
            }
            if fnd {
                (code, match val { Some(term) => term, None => Var{idx} })
            } else {
                (code, Ref{nam: nam.to_vec()})
            }
        }
    }
}

pub fn parse_env<'a>(code : &'a [u8], defs : &mut Env) -> &'a [u8] {
    if code.len() < 1 {
        code
    } else {
        match code[0] {
            // Whitespace
            b' ' => parse_env(&code[1..], defs),
            // Newline
            b'\n' => parse_env(&code[1..], defs),
            // Definition
            b'=' => {
                let (code, nam) = parse_name(&code[1..]);
                let (code, val) = parse_term(code, &mut Vec::new());
                defs.insert(nam.to_vec(), val);
                parse_env(code, defs)
            }
            _ => code
        }
    }
}

pub fn build<'a>(code : &'a [u8]) -> Env {
    let mut defs = HashMap::new();
    parse_env(code, &mut defs);
    defs
}

// Converts a source-code to a λ-term.
pub fn from_string<'a>(code : &'a [u8]) -> Term {
    let mut ctx = Vec::new();
    let (_code, term) = parse_term(code, &mut ctx);
    term
}

// Builds a var name from an index (0="a", 1="b", 26="aa"...).
pub fn var_name(idx : i32) -> Vec<u8> {
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
pub fn to_string(term : &Term, dph : i32) -> Vec<u8> {
    fn build(code : &mut Vec<u8>, term : &Term, dph : i32) {
        match term {
            &App{ref fun, ref arg} => {
                code.extend_from_slice(b":");
                build(code, &fun, dph);
                code.extend_from_slice(b" ");
                build(code, &arg, dph);
            },
            &Lam{nam: ref _nam, ref typ, ref bod} => {
                code.extend_from_slice(b"#");
                code.append(&mut var_name(dph + 1));
                code.extend_from_slice(b" ");
                build(code, &typ, dph + 1);
                code.extend_from_slice(b" ");
                build(code, &bod, dph + 1);
            },
            &All{nam: ref _nam, ref typ, ref bod} => {
                code.extend_from_slice(b"@");
                code.append(&mut var_name(dph + 1));
                code.extend_from_slice(b" ");
                build(code, &typ, dph + 1);
                code.extend_from_slice(b" ");
                build(code, &bod, dph + 1);
            },
            &Var{idx} => {
                code.append(&mut var_name(dph - idx));
            },
            &Ref{ref nam} => {
                code.append(&mut nam.clone());
            },
            //&Let{ref nam, ref val, ref bod} => {
                //code.extend_from_slice(b"=");
                //code.append(&mut nam.clone());
                //code.extend_from_slice(b" ");
                //build(code, &val, dph);
                //code.extend_from_slice(b" ");
                //build(code, &bod, dph);
            //},
            //&Bxv{ref val} => {
                //code.extend_from_slice(b"|");
                //build(code, &val, dph);
            //},
            //&Bxt{ref val} => {
                //code.extend_from_slice(b"!");
                //build(code, &val, dph);
            //},
            &Set => {
                code.extend_from_slice(b"*");
            }
        }
    }
    let mut code = Vec::new();
    build(&mut code, term, dph);
    return code;
}

// Display trait
impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&to_string(&self, 0)))
    }
}

pub fn shift(proof : &mut Term, d : i32, c : i32) {
    match proof {
        &mut App{ref mut fun, ref mut arg} => {
            shift(fun, d, c);
            shift(arg, d, c);
        },
        &mut Lam{nam: ref mut _nam, ref mut typ, ref mut bod} => {
            shift(typ, d, c+1);
            shift(bod, d, c+1);
        },
        &mut All{nam: ref mut _nam, ref mut typ, ref mut bod} => {
            shift(typ, d, c+1);
            shift(bod, d, c+1);
        },
        &mut Var{ref mut idx} => {
            *idx = if *idx < c { *idx } else { *idx + d };
        },
        //&mut Let{nam: ref mut _nam, ref mut val, ref mut bod} => {
            //shift(val, d, c);
            //shift(bod, d, c);
        //},
        &mut Ref{nam: ref mut _nam} => {},
        //&mut Bxv{ref mut val} => {
            //shift(val, d, c);
        //},
        //&mut Bxt{ref mut val} => {
            //shift(val, d, c);
        //},
        &mut Set => {}
    }
}

pub fn equals(a : &Term, b : &Term) -> bool {
    match (a,b) {
        (&App{fun: ref ay, arg: ref az},
         &App{fun: ref by, arg: ref bz})
         => equals(ay,by) && equals(az,bz),
        (&Lam{nam: ref _ay, typ: ref az, bod: ref aw},
         &Lam{nam: ref _by, typ: ref bz, bod: ref bw})
         => equals(az,bz) && equals(aw,bw),
        (&All{nam: ref _ay, typ: ref az, bod: ref aw},
         &All{nam: ref _by, typ: ref bz, bod: ref bw})
         => equals(az,bz) && equals(aw,bw),
        (&Var{idx: ref ax},
         &Var{idx: ref bx})
         => ax == bx,
        (&Ref{nam: ref ax},
         &Ref{nam: ref bx})
         => ax == bx,
        //(&Let{nam: ref _ax, val: ref ay, bod: ref az},
         //&Let{nam: ref _bx, val: ref by, bod: ref bz})
         //=> equals(ay, by) && equals(az, bz),
        //(&Bxv{val: ref ax}, &Bxv{val: ref bx})
         //=> equals(ax, bx),
        //(&Bxt{val: ref ax}, &Bxt{val: ref bx})
         //=> equals(ax, bx),
        (Set, Set)
         => true,
        _ => false
    }
}

pub fn subs(term : &mut Term, value : &Term, dph : i32) {
    let mut new_term : Option<Term> = None;
    match term {
        &mut App{ref mut fun, ref mut arg} => {
            subs(fun, value, dph);
            subs(arg, value, dph);
        },
        &mut Lam{nam: ref mut _nam, ref mut typ, ref mut bod} => {
            subs(typ, value, dph+1);
            subs(bod, value, dph+1);
        },
        &mut All{nam: ref mut _nam, ref mut typ, ref mut bod} => {
            subs(typ, value, dph+1);
            subs(bod, value, dph+1);
        },
        &mut Var{idx} => {
            if dph == idx {
                let mut val = value.clone();
                shift(&mut val, dph as i32, 0);
                new_term = Some(val);
            } else if dph < idx {
                new_term = Some(Var{idx: idx - 1})
            }
        },
        _ => {}
    };
    // Because couldn't modify Var inside its own case
    match new_term {
        Some(new_term) => *term = new_term,
        None => {}
    };
}

pub fn reduce_step(term : &mut Term, defs : &HashMap<Vec<u8>, Term>, changed : &mut bool) {
    let tmp_term = std::mem::replace(term, Set);
    let new_term : Term;
    match tmp_term {
        App{mut fun, mut arg} => {
            reduce_step(&mut fun, defs, changed);
            reduce_step(&mut arg, defs, changed);
            let tmp_fun : Term = *fun;
            match tmp_fun {
                Lam{nam: _, typ: _, mut bod} => {
                    subs(&mut bod, &arg, 0);
                    *changed = true;
                    new_term = *bod;
                },
                Ref{nam} => {
                    match defs.get(&nam) {
                        Some(val) => {
                            new_term = App{fun: Box::new(val.clone()), arg};
                            *changed = true;
                        },
                        None => new_term = Ref{nam}
                    }
                },
                t => {
                    new_term = App{fun: Box::new(t), arg};
                }
            }
        },
        Lam{nam, mut typ, mut bod} => {
            reduce_step(&mut typ, defs, changed);
            reduce_step(&mut bod, defs, changed);
            new_term = Lam{nam, typ, bod};
        },
        All{nam, mut typ, mut bod} => {
            reduce_step(&mut typ, defs, changed);
            reduce_step(&mut bod, defs, changed);
            new_term = All{nam, typ, bod};
        },
        //Let{nam, val, mut bod} => {
            //reduce_step(&mut bod, &defs.update(nam.clone(), val.clone()), changed);
            //new_term = Let{nam, val, bod};
        //},
        t => new_term = t
    }
    std::mem::replace(term, new_term);
}

pub fn reduce(term : &mut Term, defs : &HashMap<Vec<u8>, Term>) {
    let mut changed = true;
    let mut count = 0;
    while changed && count < 64 {
        count += 1;
        println!("{}", term);
        changed = false;
        reduce_step(term, defs, &mut changed);
    }
}


// A Context is a vector of (name, value) assignments.
type Context<'a> = Vec<Box<Term>>;

// Extends a context.
fn extend_context<'a>(val : Box<Term>, ctx : &'a mut Context<'a>) -> &'a mut Context<'a> {
    for i in 0..ctx.len() {
        shift(&mut ctx[i], 1, 0);
    }
    ctx.push(val);
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
pub fn infer(term : &Term, defs : &HashMap<Vec<u8>, Term>) -> Result<Term, std::string::String> {
    pub fn infer<'a>(term : &Term, ctx : &mut Context, defs : &HashMap<Vec<u8>, Term>) -> Result<Term, std::string::String> {
        match term {
            App{fun, arg} => {
                let fun_t = infer(fun, ctx, defs)?;
                let arg_t = infer(arg, ctx, defs)?;
                match fun_t {
                    All{nam: _f_nam, typ: f_typ, bod: f_bod} => {
                        let mut arg_n = arg.clone();
                        reduce(&mut arg_n, defs);
                        let mut new_typ = f_typ.clone();
                        subs(&mut new_typ, &arg_n, 0);
                        if !equals(&new_typ, &arg_t) {
                            return Err(format!("Type mismatch.\nExpected : {}\nActual   : {}", new_typ, arg_t))
                        }
                        let mut new_bod = f_bod.clone();
                        subs(&mut new_bod, &arg_n, 0);
                        reduce(&mut new_bod, defs);
                        Ok(*new_bod)
                    },
                    _ => {
                        Err("Non-function application.".to_string())
                    }
                }
            },
            Lam{nam, typ, bod} => {
                let mut typ_n = typ.clone();
                reduce(&mut typ_n, defs);
                extend_context(typ_n.clone(), ctx);
                let bod_t = Box::new(infer(bod, ctx, defs)?);
                narrow_context(ctx);
                Ok(All{nam: nam.clone(), typ: typ_n, bod: bod_t})
            },
            All{nam: _, typ, bod} => {
                let mut typ_n = typ.clone();
                reduce(&mut typ_n, defs);
                extend_context(typ_n, ctx);
                let typ_t = Box::new(infer(typ, ctx, defs)?);
                let bod_t = Box::new(infer(bod, ctx, defs)?);
                narrow_context(ctx);
                if equals(&typ_t, &bod_t) {
                    Ok(Set)
                } else {
                    Err("Forall not a type.".to_string())
                }
            },
            Var{idx} => {
                Ok(*ctx[ctx.len() - (*idx as usize) - 1].clone())
            },
            Ref{nam: _} => {
                Err("TODO".to_string())
            },
            Set => {
                Ok(Set)
            },
            //Let{nam:_, val:_, bod:_} => {
                //panic!("todo")
            //}
        }
    }
    let mut ctx : Vec<Box<Term>> = Vec::new();
    infer(term, &mut ctx, defs)
}


