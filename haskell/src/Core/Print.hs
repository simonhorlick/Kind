-- this module contains orphan instances and is designed to be imported
-- qualified as `import qualified Core.Print as Core.Print`
module Core.Print where

import           Data.Map  (Map)
import qualified Data.Map  as M
import           Data.Text (Text)
import qualified Data.Text as T hiding (find)
import           Prelude   hiding (print)

import           Core.Rig
import           Core.Type

term :: Term -> Text
term t = let go = term in case t of
  Var _ n _            -> n
  Ref _ n              -> n
  Typ _                -> "*"
  All _ r s n h b ->
    let body = go (b (Var noLoc (T.snoc s '#') 0) (Var noLoc (T.snoc n '#') 0))
        self = if s == "" then "" else T.concat ["$(", s, "). "]
     in case r of
          Zero -> T.concat [self, "∀(", n, ":", go h, "). ", body]
          One  -> T.concat [self, "(", n, ":", go h, ") ⊸ ", body]
          Many -> T.concat [self, "(", n, ":", go h, ") → ", body]
  Lam _ n b          ->
    let body = go (b (Var noLoc (T.snoc n '#') 0))
    in T.concat ["λ", n, ". ", body]
  App _ f a      -> T.concat ["(", go f, " ", go a, ")"]
  Let _ n x b        -> let body = go (b (Var noLoc (T.snoc n '#') 0)) in
    T.concat ["let ", n, "=", go x, ";", body]
  Ann _ d x t          -> T.concat [":", go t, " ", go x]

instance Show Term where
  show x = T.unpack (Core.Print.term x)

expr :: Expr -> Text
expr (Expr n typ trm) = T.concat
  [ n, ": ", Core.Print.term typ,"\n  " , Core.Print.term trm]

instance Show Expr where
  show x = T.unpack (Core.Print.expr x)

modl :: Module -> Text
modl (Module defs) = go $ snd <$> M.toList defs
  where
    go [] = ""
    go (x:[]) = Core.Print.expr x
    go (x:xs) = T.concat [Core.Print.expr x, "\n", go xs]

instance Show Module where
  show x = T.unpack (Core.Print.modl x)
