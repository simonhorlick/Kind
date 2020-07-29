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

reduce :: Term -> Module -> Bool -> Term
reduce term (Module defs) erase = go term
  where
    go term = --trace ("reduce: " ++ show term) $ 
      case term of
        Var _ n idx      ->
          -- trace (T.unpack $ "Var " `T.append` n) $ 
          Var noLoc n idx
        Ref l n          ->
          --trace (T.unpack $ "Ref " `T.append` n) $ 
          case defs Map.!? n of
            Just (Expr _ _ got@(Ref _ m)) -> got
            Just (Expr _ _ got)           -> go got
            Nothing                       -> Ref l n
        Typ _            -> {-trace "reduce Type" $-} Typ noLoc
        All _ _ _ _ _ _  -> {-trace "reduce All"  $-} term
        Lam _ e n b      ->
          -- trace "reduce Lam"  $ 
          if e && erase then go (Lam noLoc False "" (\x -> x)) else term
        App _ e f a      -> 
          -- trace "reduce App" $ 
          if e && erase then go f else
            case go f of
              Lam _ e n b  -> -- trace ("App Lam: " ++ show f) $ 
                go (b a)
              x          -> -- trace ("App f: " ++ show f) $ 
                term
        Let _ n x b      -> go (b x)
        Ann _ _ x t      -> go x


-- Normalize
normalize :: Term -> Module -> Bool -> Term
normalize term defs erased = runST (top term)
  where
    top :: Term -> ST s Term
    top term = do
      seen <- newSTRef (Set.empty)
      -- traceM "top"
      go term seen

    go :: Term -> (STRef s (Set Hash)) -> ST s Term
    go term seen = {- trace "go" $ -} do
      let norm  = reduce term defs erased
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
             All _ e s n h b  -> {- trace "norm All" $ -} do
               bind <- go h seen
               return $ All noLoc e s n bind (\s x -> unsafePerformST $ go (b s x) seen)
             Lam _ e n b      -> trace "norm Lam" $ traceShow norm $ do
               return $ Lam noLoc e n (\x -> unsafePerformST $ go (b x) seen)
             App _ e f a      -> trace "norm App" $ do
              func <- go f seen
              argm <- go a seen
              return $ App noLoc e func argm
             Let _ n x b      -> trace "norm Let" $ go (b x) seen
             Ann _ _ x t      -> trace "norm Ann" $ go x seen

