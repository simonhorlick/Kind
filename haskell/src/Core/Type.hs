module Core.Type where

import Core.Rig

import Data.Text (Text)
import qualified Data.Text as T hiding (find)

import Data.Map (Map)
import qualified Data.Map as M

type Name = Text

data Term
  = Var Loc Text Integer
  | Ref Loc Text
  | Typ Loc
  | All Loc Rig  Name Name Term (Term -> Term -> Term)
  | Lam Loc Name (Term -> Term)
  | App Loc Term Term
  | Let Loc Name Term (Term -> Term)
  | Ann Loc Bool Term Term

data Loc = Loc { _from :: Int, _upto :: Int } deriving Show

noLoc = Loc 0 0

data Expr   = Expr { _name :: Name, _type :: Term, _term :: Term }
data Module = Module { _defs :: (Map Name Expr)}

deref :: Name -> Module -> Expr
deref n m = (_defs m) M.! n

type Ctx = [(Name,Term)]

find :: Ctx -> ((Name,Term) -> Bool) -> Maybe ((Name,Term),Int)
find ctx f = go ctx 0
  where
    go [] _     = Nothing
    go (x:xs) i = if f x then Just (x,i) else go xs (i+1)
