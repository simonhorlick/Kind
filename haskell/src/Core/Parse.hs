module Core.Parse where

import           Control.Monad.Identity

import           Data.Text                  (Text)
import qualified Data.Text                  as T
import           Data.Map (Map)
import qualified Data.Map as M
import           Data.Void

import           Text.Megaparsec            hiding (State)
import           Text.Megaparsec.Char
import qualified Text.Megaparsec.Char.Lexer as L

import           Core.Rig
import           Core.Type
import qualified Core.Print as Core.Print

type Parser = ParsecT Void Text Identity

name :: Bool -> Parser Name
name empty = (if empty then takeWhileP else takeWhile1P)
  (Just "a name (alphanumeric,'_','.')") (flip elem $ nameChar)
  where nameChar = ['0'..'9'] ++ ['a'..'z'] ++ ['A'..'Z'] ++ "_" ++ "."

spaceC :: Parser ()
spaceC = L.space space1 (L.skipLineComment "//") (L.skipBlockComment "#" "#")

symbol :: Text -> Parser Text
symbol = L.symbol spaceC

-- The term parser
-- TODO: proper error printing instead of the `\n` hack to
-- make the ShowErrorComponent print nicely
-- (Perhaps by using a typerep proxy plus a new typeclass instance)
term :: Parser (Ctx -> Term)
term = do
  from <- getOffset
  t    <- choice $
    [ label "\n - the type of types: \"*\"" $
        symbol "*" >> (return $ \ctx -> Typ (Loc from (from+1)))
    , label "\n - a forall: \"Πself(n x: A) B\"" $ do
        string "Π"
        self <- name True <* symbol "("
        rig  <- (string "0" *> return Zero) <|> (string "1" *> return One) <|> (string "ω" *> return Many)
        name <- name True <* spaceC <* symbol ":"
        bind <- term <* symbol ")"
        body <- term
        upto <- getOffset
        return $ \ctx -> All (Loc from upto)
          rig self name (bind ctx) (\s x -> body ((name,x):(self,s):ctx))
    , label "\n - a lambda: \"λx b\"" $ do
        from <- getOffset
        string "λ"
        name <- name True <* spaceC
        body <- term
        upto <- getOffset
        return $ \ctx ->
          Lam (Loc from upto) name (\x -> body ((name,x):ctx))
    , label "\n - an application: \"(f a)\"" $ do
        symbol "("
        func <- term
        argm <- term
        symbol ")"
        upto <- getOffset
        return $ \ctx ->
          App (Loc from upto) (func ctx) (argm ctx)
    , label "\n - a definition: \"$x = y; b\", \"@x = y; b\"" $ do
        string "$"
        name <- name True <* spaceC <* symbol "="
        expr <- term <* symbol ";"
        body <- term
        upto <- getOffset
        return $ \ctx ->
          Let (Loc from upto) name (expr ctx) (\x -> body ((name,x):ctx))
    , label "\n - a type annotation: \":A x\"" $ do
        symbol ":"
        typ_ <- term
        expr <- term
        upto <- getOffset
        return $ \ctx ->
          Ann (Loc from upto) False (expr ctx) (typ_ ctx)
    , label "\n - a reference, either global or local: \"x\"" $ do
        name <- name False
        upto <- getOffset
        return $ \ctx -> case find ctx (\(n,i) -> n == name) of
          Nothing -> Ref (Loc from upto) name
          Just ((n,t),i) -> t
    ]
  spaceC
  return t

expr :: Parser Expr
expr = do
  name <- name False <* symbol ":"
  typ_ <- ($ []) <$> term
  term <- ($ []) <$> term
  return $ Expr name typ_ term

modl :: Parser Module
modl = Module . M.fromList . fmap (\d -> (_name d, d)) <$> defs
  where
    defs :: Parser [Expr]
    defs = (do {spaceC; x <- expr; (x:) <$> defs}) <|> (spaceC >> return [])
