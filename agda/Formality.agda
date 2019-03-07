module Formality where

open import Data.Nat
open import Data.Fin hiding (_+_)

data Term : (n : ℕ) → Set where
  typ : ∀ {d} → Term d
  var : ∀ {d} → (i : Fin d) → Term d
  all : ∀ {d} → Term (suc d) → Term (suc d) → Term d
  lam : ∀ {d} → Term (suc d) → Term (suc d) → Term d
  app : ∀ {d} → Term d → Term d → Term d

-- A table renaming every `Fin n` by a `Fin m`
Niks : (n m : ℕ) → Set
Niks n m = Fin n → Fin m

-- Increases all renamings in a table, not renaming zero
inc : ∀ {n m} → (niks : Niks n m) → Niks (suc n) (suc m)
inc niks zero    = zero
inc niks (suc i) = suc (niks i)

-- Renames every free var in a `Term n`
rename : ∀ {n m} → (niks : Niks n m) (t : Term n) → Term m
rename niks typ           = typ
rename niks (var idx)     = var (niks idx)
rename niks (all bin bod) = all (rename (inc niks) bin) (rename (inc niks) bod)
rename niks (lam bin bod) = lam (rename (inc niks) bin) (rename (inc niks) bod)
rename niks (app fun arg) = app (rename niks fun) (rename niks arg)

-- A substitution table for terms with `n` free vars into terms of `m` free vars
Subs : (n m : ℕ) → Set
Subs n m = Fin n → Term m

-- Shifts all substitutions, not substituting (var 0)
shi : ∀ {n m} → Subs n m → Subs (suc n) (suc m)
shi ctx zero    = var zero
shi ctx (suc l) = rename suc (ctx l)

-- Substitutes `n` free vars by terms of `m` free vars, returning `Term m`
subst : ∀ {n m} → (subs : Subs n m) (t : Term n) → Term m
subst subs typ           = typ
subst subs (var i)       = subs i
subst subs (all bin bod) = all (subst (shi subs) bin) (subst (shi subs) bod)
subst subs (lam bin bod) = lam (subst (shi subs) bin) (subst (shi subs) bod)
subst subs (app fun arg) = app (subst subs fun) (subst subs arg)

-- Contexts
Γ : (n : ℕ) → Set
Γ n = Fin n → Term n

-- Empty context
ε : Fin 0 → Term 0
ε = λ ()

-- Extends the context
_,_ : ∀ {n} → Γ n → Term (suc n) → Γ (suc n)
_,_ {n} ctx ty zero    = ty
_,_ {n} ctx ty (suc l) = rename suc (ctx l)

-- Beta equality
data _~_ {n} : Term n → Term n → Set where
  ~β     : ∀ b f x → app (lam b f) x ~ subst (λ v → x) f
  ~app   : ∀ t t' u u' → t ~ t' → u ~ u' → app t u ~ app t' u'
  ~lam   : ∀ b b' t t' → b ~ b' → t ~ t' → lam b t ~ lam b' t'
  ~refl  : ∀ t → t ~ t
  ~sym   : ∀ t t' → t ~ t' → t' ~ t
  ~trans : ∀ t t' t'' → t ~ t' → t' ~ t'' → t ~ t''

-- Typing relation
data _⊢_∷_ {n} : Γ n → Term n → Term n → Set where
  typ-ty : ∀ {Γ} →
    Γ ⊢ typ ∷ typ
  var-ty : ∀ {Γ i} →
    Γ ⊢ var i ∷ Γ i
  lam-ty : ∀ {Γ bind body &body} →
    (Γ , bind) ⊢ body ∷ &body →
    Γ ⊢ all bind &body ∷ typ →
    Γ ⊢ lam bind body ∷ all bind &body
  all-ty : ∀ {Γ bind body} →
    (Γ , bind) ⊢ bind ∷ typ →
    (Γ , bind) ⊢ body ∷ typ →
    Γ ⊢ all bind body ∷ typ
  app-ty : ∀ {Γ func argm func-bind func-body} →
    Γ ⊢ func ∷ all func-bind func-body →
    Γ ⊢ argm ∷ subst (λ x → argm) func-bind →
    Γ ⊢ app func argm ∷ subst (λ x → argm) func-body
  eql-ty : ∀ {Γ a t t'} →
    Γ ⊢ a ∷ t → t ~ t' → Γ ⊢ a ∷ t'

-- one-step reduction
data _~>_ {n} : Term n → Term n → Set where
  β    : ∀ bind body argm → app (lam bind body) argm ~> subst (λ k → argm) body
  app₁ : ∀ func func' argm → func ~> func' → app func argm ~> app func' argm
  app₂ : ∀ func argm argm' → argm ~> argm' → app func argm ~> app func argm'
  lam₁ : ∀ bind bind' body → bind ~> bind' → lam bind body ~> lam bind' body
  lam₂ : ∀ bind body body' → body ~> body' → lam bind body ~> lam bind body'

-- Preservation

preservation : ∀ (n : ℕ) → (Γ : Γ n) → (t : Term n) → (t' : Term n) → (T : Term n) → t ~> t' → Γ ⊢ t ∷ T → Γ ⊢ t' ∷ T
preservation n Γ .(app (lam bind body) argm) .(subst (λ k → argm) body) T (β bind body argm) an = {!   !}
preservation n Γ .(app func argm) .(app func' argm) T (app₁ func func' argm rx) an = {!   !}
preservation n Γ .(app func argm) .(app func argm') T (app₂ func argm argm' rx) an = {!   !}
preservation n Γ .(lam typ body) .(lam bind' body) T (lam₁ typ bind' body rx) an = {!   !}
preservation n Γ .(lam (var i) body) .(lam bind' body) T (lam₁ (var i) bind' body rx) an = {!   !}
preservation n Γ .(lam (all bind bind₁) body) .(lam bind' body) T (lam₁ (all bind bind₁) bind' body rx) an = {!   !}
preservation n Γ .(lam (lam bind bind₁) body) .(lam bind' body) T (lam₁ (lam bind bind₁) bind' body rx) an = {!   !}
preservation n Γ .(lam (app bind bind₁) body) .(lam bind' body) T (lam₁ (app bind bind₁) bind' body rx) an = {!   !}
preservation n Γ .(lam bind body) .(lam bind body') T (lam₂ bind body body' rx) an = {!   !}


-- Closed typing relation
_∷_ : Term zero → Term zero → Set
_∷_ a b = ε ⊢ a ∷ b

-- {x : Type} Type ∷ Type
t0 : all typ typ ∷ typ
t0 = all-ty typ-ty typ-ty

-- [x : Type] x ∷ {x : Type} Type
t1 : lam typ (var zero) ∷ all typ typ
t1 = lam-ty var-ty t0
