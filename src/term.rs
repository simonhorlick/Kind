use std;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Term {

    // Forall
    All {
        nam: Vec<u8>,
        typ: Box<Term>,
        bod: Box<Term>
    },

    // Lambda
    Lam {
        nam: Vec<u8>,
        typ: Box<Term>,
        bod: Box<Term>
    },

    // Variable
    Var {
        idx: i32
    },

    // Application
    App {
        fun: Box<Term>,
        arg: Box<Term>
    },

    // Inductive Data Type
    Idt {
        nam: Vec<u8>,
        typ: Box<Term>,
        ctr: Vec<(Vec<u8>, Box<Term>)>
    },

    // Constructor
    Ctr {
        nam: Vec<u8>,
        idt: Box<Term>
    },

    // Pattern-Matching
    Cas {
        idt: Box<Term>,
        val: Box<Term>,
        ret: Box<Term>,
        cas: Vec<(Vec<u8>, Box<Term>)>
    },

    // Type of Types
    Set
}

use self::Term::{*};

// Note: parsing currently panics on error. TODO: chill and return a result.

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

// Skips spaces, newlines, etc.
pub fn skip_whites(code : &[u8]) -> &[u8] {
    let mut new_code : &[u8] = code;
    while new_code.len() > 0 && new_code[0] == b' ' || new_code[0] == b'\n' {
        new_code = &new_code[1..];
    }
    new_code
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
    let code = skip_whites(code);
    match code[0] {
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
        // Inductive Data Type
        b'$' => {
            let (code, nam) = parse_name(&code[1..]);
            let (code, typ) = parse_term(code, ctx);
            extend_scope(nam, None, ctx);
            let nam = nam.to_vec();
            let typ = Box::new(typ);
            let code = skip_whites(code);
            let mut new_code = code;
            let mut ctr : Vec<(Vec<u8>, Box<Term>)> = Vec::new();
            while new_code.len() > 0 && new_code[0] == b'|' {
                let code = &new_code[1..];
                let (code, ctr_nam) = parse_name(code);
                println!("found {}", String::from_utf8_lossy(ctr_nam));
                let (code, ctr_typ) = parse_term(code, ctx);
                let code = skip_whites(code);
                let ctr_nam = ctr_nam.to_vec();
                let ctr_typ = Box::new(ctr_typ);
                ctr.push((ctr_nam, ctr_typ));
                new_code = code;
            };
            narrow_scope(ctx);
            (new_code, Idt{nam, typ, ctr})
        },
        // Constructor
        b'.' => {
            let (code, nam) = parse_name(&code[1..]);
            let (code, idt) = parse_term(code, ctx);
            let nam = nam.to_vec();
            let idt = Box::new(idt);
            (code, Ctr{nam,idt})
        },
        // Pattern-Matching
        b'~' => {
            let (code, idt) = parse_term(&code[1..], ctx);
            let (code, val) = parse_term(code, ctx);
            extend_scope(b"self", None, ctx);
            let (code, ret) = parse_term(code, ctx);
            narrow_scope(ctx);
            let val = Box::new(val);
            let idt = Box::new(idt);
            let ret = Box::new(ret);
            let code = skip_whites(code);
            let mut new_code = code;
            let mut cas : Vec<(Vec<u8>, Box<Term>)> = Vec::new();
            println!("go match");
            while new_code.len() > 0 && new_code[0] == b'|' {
                println!("found");
                let code = &new_code[1..];
                let (code, cas_nam) = parse_name(code);
                let (code, cas_fun) = parse_term(code, ctx);
                let code = skip_whites(code);
                let cas_nam = cas_nam.to_vec();
                let cas_fun = Box::new(cas_fun);
                cas.push((cas_nam, cas_fun));
                new_code = code;
            }
            (code, Cas{idt,val,ret,cas})
        },
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
                panic!("Unbound variable: {}", String::from_utf8_lossy(nam));
            }
        }
    }
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
            &Idt{nam: _, ref typ, ref ctr} => {
                code.extend_from_slice(b"$");
                code.append(&mut var_name(dph + 1));
                code.extend_from_slice(b" ");
                build(code, &typ, dph + 1);
                for (nam,typ) in ctr {
                    code.extend_from_slice(b" ");
                    code.extend_from_slice(b"|");
                    code.append(&mut nam.clone());
                    code.extend_from_slice(b" ");
                    build(code, &typ, dph + 1);
                }
            },
            &Ctr{ref nam, ref idt} => {
                code.extend_from_slice(b".");
                code.append(&mut nam.clone());
                code.extend_from_slice(b" ");
                build(code, &idt, dph+1);
            },
            &Cas{ref idt, ref val, ref ret, ref cas} => {
                code.extend_from_slice(b"~");
                build(code, &idt, dph);
                code.extend_from_slice(b" ");
                build(code, &val, dph);
                code.extend_from_slice(b" ");
                build(code, &ret, dph + 1);
                for (nam,fun) in cas {
                    code.extend_from_slice(b" ");
                    code.extend_from_slice(b"|");
                    code.append(&mut nam.clone());
                    code.extend_from_slice(b" ");
                    build(code, &fun, dph + 1);
                }
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

pub fn shift(term : &mut Term, d : i32, c : i32) {
    match term {
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
        &mut Idt{nam: ref mut _nam, ref mut typ, ref mut ctr} => {
            shift(typ, d, c);
            for (_,ctr_typ) in ctr {
                shift(ctr_typ, d, c+1);
            }
        },
        &mut Ctr{nam: ref mut _nam, ref mut idt} => {
            shift(idt, d, c);
        },
        &mut Cas{ref mut idt, ref mut val, ref mut ret, ref mut cas} => {
            shift(idt, d, c);
            shift(val, d, c);
            shift(ret, d, c+1);
            for (_,cas_fun) in cas {
                shift(cas_fun, d, c);
            }
        },
        //&mut Let{nam: ref mut _nam, ref mut val, ref mut bod} => {
            //shift(val, d, c);
            //shift(bod, d, c);
        //},
        //&mut Bxv{ref mut val} => {
            //shift(val, d, c);
        //},
        //&mut Bxt{ref mut val} => {
            //shift(val, d, c);
        //},
        &mut Set => {}
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
        &mut Idt{nam: ref mut _nam, ref mut typ, ref mut ctr} => {
            subs(typ, value, dph);
            for (_,ctr_typ) in ctr {
                subs(ctr_typ, value, dph+1);
            }
        },
        &mut Ctr{nam: _, ref mut idt} => {
            subs(idt, value, dph);
        },
        &mut Cas{ref mut idt, ref mut val, ref mut ret, ref mut cas} => {
            subs(idt, value, dph);
            subs(val, value, dph);
            subs(ret, value, dph);
            for (_,cas_fun) in cas {
                subs(cas_fun, value, dph+1);
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

pub fn equals(a : &Term, b : &Term) -> bool {
    match (a,b) {
        (&App{fun: ref a_fun, arg: ref a_arg}, &App{fun: ref b_fun, arg: ref b_arg}) => {
            equals(a_fun, b_fun) && equals(a_arg, b_arg)
        },
        (&Lam{nam: _, typ: ref a_typ, bod: ref a_bod}, &Lam{nam: _, typ: ref b_typ, bod: ref b_bod}) => {
            equals(a_typ, b_typ) && equals(a_bod, b_bod)
        },
        (&All{nam: _, typ: ref a_typ, bod: ref a_bod}, &All{nam: _, typ: ref b_typ, bod: ref b_bod}) => {
            equals(a_typ, b_typ) && equals(a_bod, b_bod)
        },
        (&Var{idx: ref a_idx}, &Var{idx: ref b_idx}) => {
            a_idx == b_idx
        },
        (&Idt{nam: _, typ: ref a_typ, ctr: ref a_ctr}, &Idt{nam: _, typ: ref b_typ, ctr: ref b_ctr}) => {
            let mut eql_ctr = true;
            for i in 0..a_ctr.len() {
                let (_, a_ctr_typ) = a_ctr[i].clone();
                let (_, b_ctr_typ) = b_ctr[i].clone();
                eql_ctr = eql_ctr && equals(&a_ctr_typ, &b_ctr_typ);
            }
            equals(a_typ, b_typ) && eql_ctr
        },
        (&Ctr{nam: _, idt: ref a_idt}, &Ctr{nam:_, idt: ref b_idt}) => {
            equals(a_idt, b_idt)
        },
        (&Cas{idt: ref a_idt, val: ref a_val, ret: ref a_ret, cas: ref a_cas}, &Cas{idt: ref b_idt, val: ref b_val, ret: ref b_ret, cas: ref b_cas}) => {
            let mut eql_cas = true;
            for i in 0..a_cas.len() {
                let (_, a_cas_fun) = a_cas[i].clone();
                let (_, b_cas_fun) = b_cas[i].clone();
                eql_cas = eql_cas && equals(&a_cas_fun, &b_cas_fun);
            }
            equals(a_idt, b_idt) && equals(a_val, b_val) && equals(a_ret, b_ret) && eql_cas
        },

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

fn pattern_match(val : &mut Term, cases : &Vec<(Vec<u8>,Box<Term>)>) -> bool {
    println!("UE {}", cases.len());
    let tmp_val = std::mem::replace(val, Set);
    let new_val : Term;
    let changed = match tmp_val {
        App{mut fun, arg} => {
            let changed = pattern_match(&mut fun, cases);
            new_val = App{fun, arg};
            changed
        },
        Ctr{nam, idt:_} => {
            println!("yay {}", String::from_utf8_lossy(&nam));
            let mut fun : Term = Set;
            for i in 0..cases.len() {
                let case_nam = &cases[i].0;
                println!("woa {}", String::from_utf8_lossy(&case_nam));
                let case_fun = &cases[i].1;
                if *case_nam == nam {
                    fun = *case_fun.clone();
                }
            }
            new_val = fun;
            true
        },
        t => {
            new_val = t;
            false
        }
    };
    std::mem::replace(val, new_val);
    changed
}

pub fn make_case_fun(fun : &Term, idt : &Term, ret : &Term, mut val : Term) -> Term {
    //data Foo : Nat -> Set where
        //MkFoo : (x : Nat) -> (f : Foo (suc x)) -> Foo (suc x)
    //CTOR => (x : Nat) -> (t : Foo (suc x)) -> Foo (suc x)
    //CASE => (x : Nat) -> (t : Foo (suc x)) -> P (MkFoo x t)
    match fun {
        All{nam, ref typ, ref bod} => {
            let mut typ = typ.clone();
            subs(&mut typ, idt, 0);
            shift(&mut val, 1, 0);
            val = App{fun: Box::new(val), arg: Box::new(Var{idx: 0})};
            let bod = make_case_fun(bod, idt, ret, val);
            let nam = nam.to_vec();
            let bod = Box::new(bod);
            All{nam, typ, bod}
        },
        t => {
            let mut ret : Term = ret.clone();
            subs(&mut ret, &val, 0);
            ret
        }
    }
}

pub fn reduce_step(term : &mut Term, changed : &mut bool) {
    let tmp_term = std::mem::replace(term, Set);
    let new_term : Term;
    match tmp_term {
        App{mut fun, mut arg} => {
            reduce_step(&mut fun, changed);
            reduce_step(&mut arg, changed);
            let tmp_fun : Term = *fun;
            match tmp_fun {
                Lam{nam: _, typ: _, mut bod} => {
                    subs(&mut bod, &arg, 0);
                    *changed = true;
                    new_term = *bod;
                },
                t => {
                    new_term = App{fun: Box::new(t), arg};
                }
            }
        },
        Lam{nam, mut typ, mut bod} => {
            reduce_step(&mut typ, changed);
            reduce_step(&mut bod, changed);
            new_term = Lam{nam, typ, bod};
        },
        All{nam, mut typ, mut bod} => {
            reduce_step(&mut typ, changed);
            reduce_step(&mut bod, changed);
            new_term = All{nam, typ, bod};
        },
        Idt{nam, mut typ, mut ctr} => {
            reduce_step(&mut typ, changed);
            for i in 0..ctr.len() {
                reduce_step(&mut ctr[i].1, changed);
            }
            new_term = Idt{nam, typ, ctr};
        },
        Ctr{nam, mut idt} => {
            reduce_step(&mut idt, changed);
            new_term = Ctr{nam, idt};
        },
        Cas{mut idt, mut val, mut ret, mut cas} => {
            reduce_step(&mut idt, changed);
            reduce_step(&mut val, changed);
            reduce_step(&mut ret, changed);
            for i in 0..cas.len() {
                let cas_fun = &mut cas[i].1;
                reduce_step(cas_fun, changed);
            }
            if pattern_match(&mut val, &cas) {
                new_term = *val;
                *changed = true;
            } else {
                new_term = Cas{idt, val, ret, cas};
            }
        },
        t => new_term = t
    }
    std::mem::replace(term, new_term);
}

pub fn reduce(term : &mut Term) {
    let mut changed = true;
    let mut count = 0;
    while changed && count < 64 {
        count += 1;
        changed = false;
        reduce_step(term, &mut changed);
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
pub fn infer(term : &Term) -> Result<Term, std::string::String> {
    pub fn infer<'a>(term : &Term, ctx : &mut Context) -> Result<Term, std::string::String> {
        match term {
            App{fun, arg} => {
                let fun_t = infer(fun, ctx)?;
                let arg_t = infer(arg, ctx)?;
                match fun_t {
                    All{nam: _f_nam, typ: f_typ, bod: f_bod} => {
                        let mut arg_n = arg.clone();
                        reduce(&mut arg_n);
                        let mut new_typ = f_typ.clone();
                        subs(&mut new_typ, &arg_n, 0);
                        if !equals(&new_typ, &arg_t) {
                            return Err(format!("Type mismatch.\nExpected : {}\nActual   : {}", new_typ, arg_t))
                        }
                        let mut new_bod = f_bod.clone();
                        subs(&mut new_bod, &arg_n, 0);
                        reduce(&mut new_bod);
                        Ok(*new_bod)
                    },
                    _ => {
                        Err("Non-function application.".to_string())
                    }
                }
            },
            Lam{nam, typ, bod} => {
                let mut typ_n = typ.clone();
                reduce(&mut typ_n);
                extend_context(typ_n.clone(), ctx);
                let bod_t = Box::new(infer(bod, ctx)?);
                narrow_context(ctx);
                Ok(All{nam: nam.clone(), typ: typ_n, bod: bod_t})
            },
            All{nam: _, typ, bod} => {
                let mut typ_n = typ.clone();
                reduce(&mut typ_n);
                extend_context(typ_n, ctx);
                let typ_t = Box::new(infer(typ, ctx)?);
                let bod_t = Box::new(infer(bod, ctx)?);
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
            Idt{nam: _, typ, ctr: _} => {
                let mut typ_v = typ.clone();
                reduce(&mut typ_v);
                Ok(*typ_v)
            },
            Ctr{nam, idt} => {
                match **idt {
                    Idt{nam:_, typ: _, ref ctr} => {
                        for i in 0..ctr.len() {
                            let ctr_nam = &ctr[i].0;
                            let ctr_typ = &ctr[i].1;
                            if ctr_nam == nam {
                                let mut res_typ = ctr_typ.clone();
                                subs(&mut res_typ, &idt.clone(), 0);
                                return Ok(*res_typ);
                            }
                        }
                        return Err(format!("Constructor not found: {}.", String::from_utf8_lossy(nam)))
                    },
                    _ => Err(format!("Not an IDT: {}", idt))
                }
            },
            Cas{idt, val, ret, cas} => {
                panic!("vish");
                let tmp_idt : &Term = idt;
                match tmp_idt {
                    Idt{nam:_, typ, ctr} => {
                        if cas.len() != ctr.len() {
                            return Err(format!("Mismatched pattern-match."));
                        } 
                        for i in 0..ctr.len() {
                            let ctr_nam = &ctr[i].0;
                            let ctr_typ = &ctr[i].1;
                            let cas_nam = &cas[i].0;
                            let cas_fun = &cas[i].1;
                                //pub fn make_case_fun(fun : &Term, idt : &Term, ret : &Term, mut val : Term) -> Term {
                            if ctr_nam != cas_nam {
                                return Err(format!("Mismatched pattern-match."));
                            }
                            let ctr_val = Ctr{nam: ctr_nam.to_vec(), idt: *idt};
                            let fun_typ = make_case_fun(ctr_typ, idt, ret, ctr_val);
                            //let mut expect_typ = ctr_typ.clone();
                            //subs(&mut expect_typ, ret
                        }
                        panic!("ok");
                    },
                    _ => Err(format!("Not an IDT: {}", &idt))
                }
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
    infer(term, &mut ctx)
}

































