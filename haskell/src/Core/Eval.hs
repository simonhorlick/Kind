{-# LANGUAGE OverloadedStrings #-}
module Core.Eval where

import Core.Type
import Core.Hash

import Data.Map (Map)
import qualified Data.Map as Map

import Data.Set (Set)
import qualified Data.Set as Set

import           Data.Text                  (Text)
import qualified Data.Text                  as T

import Control.Monad.ST
import Data.STRef
import Control.Monad.ST.UnsafePerform

import Debug.Trace

reduce :: Term -> Module -> Term
reduce term mod@(Module defs) = case term of
  Var _ n idx      ->
    Var noLoc n idx
  Ref l n          ->
    case defs Map.!? n of
      -- Just (Expr _ _ got@(Ref _ m)) -> got
      Just (Expr _ _ got)           -> reduce got mod
      Nothing                       -> Ref l n
  Typ _            -> Typ noLoc
  All _ _ _ _ _ _  -> term
  Lam _ n b        -> term
  App _ f a        -> 
    case reduce f mod of
      Lam _ n b -> reduce (b a) mod
      x         -> term
  Let _ n x b      -> reduce (b x) mod
  Ann _ _ x t      -> reduce x mod


-- Normalize
normalize :: Term -> Module -> Term
normalize term defs = runST (top term)
  where
    top :: Term -> ST s Term
    top term = do
      seen <- newSTRef (Set.empty)
      -- traceM "top"
      go term seen

    go :: Term -> (STRef s (Set Hash)) -> ST s Term
    go term seen = {- trace "go" $ -} do
      let norm  = reduce term defs
      let termH = hash term
      let normH = hash norm
      -- traceM $ concat ["term: ",show term, " hash: ",show termH]
      -- traceM $ concat ["norm: ",show norm, " hash: ",show normH]
      seen' <- readSTRef seen
      -- traceM $ concat ["seen: ",show seen']
      if | (termH `Set.member` seen' || normH `Set.member` seen') -> return norm
         | otherwise -> do
           modifySTRef' seen ((Set.insert termH) . (Set.insert normH))
           case norm of
             Var l n idx      -> {- trace "norm Var" $ -} return $ Var l n idx
             Ref l n          -> {- trace "norm Ref" $ -} return $ Ref l n
             Typ l            -> {- trace "norm Typ" $ -} return $ Typ l
             All _ r s n h b  -> {- trace "norm All" $ -} do
               bind <- go h seen
               return $ All noLoc r s n bind (\s x -> unsafePerformST $ go (b s x) seen)
             Lam _ n b        -> trace "norm Lam" $ traceShow norm $ do
               return $ Lam noLoc n (\x -> unsafePerformST $ go (b x) seen)
             App _ f a        -> trace "norm App" $ do
              func <- go f seen
              argm <- go a seen
              return $ App noLoc func argm
             -- Should not happen
             Let _ n x b      -> trace "norm Let" $ go (b x) seen
             Ann _ _ x t      -> trace "norm Ann" $ go x seen

