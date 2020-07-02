 -- {-# LANGUAGE RankNTypes #-}
 -- {-# LANGUAGE ScopedTypeVariables #-}
 -- {-# LANGUAGE TypeApplications #-}
module Core.Check where

import Core.Type
import Core.Hash
import Core.Eval

import Data.Map (Map)
import qualified Data.Map as Map

import Data.Set (Set)
import qualified Data.Set as Set

import           Data.Text                  (Text)
import qualified Data.Text                  as T

import Control.Monad.ST
import Control.Monad.Except
import Data.STRef

import Debug.Trace


-- TODO: share set of equals hashes between `equal` calls?
equal :: Term -> Term -> Module -> Int -> Bool
equal a b defs dep = runST $ top a b dep
  where
    top :: Term -> Term -> Int -> ST s Bool
    top a b dep = do
      seen <- newSTRef (Set.empty)
      go a b dep seen

    go :: Term -> Term -> Int -> STRef s (Set (Hash,Hash)) -> ST s Bool
    go a b dep seen = do
      let var d = Var noLoc (T.cons '#' $ T.pack $ show d)
      let a1 = reduce a defs True
      let b1 = reduce b defs True
      let ah = hash a1
      let bh = hash b1
      -- traceM $ show a
      -- traceM $ show b
      s' <- readSTRef seen
      if | (ah == bh)              -> return True
         | (ah,bh) `Set.member` s' -> return True
         | (bh,ah) `Set.member` s' -> return True
         | otherwise -> do
             modifySTRef' seen ((Set.insert (ah,bh)) . (Set.insert (bh,ah)))
             case (a1,b1) of
               (All _ ae as _ ah ab, All _ be bs _ bh bb) -> do
                 let a1_body = ab (var dep) (var (dep + 1))
                 let b1_body = bb (var dep) (var (dep + 1))
                 let eras_eq = ae == be
                 let self_eq = as == bs
                 bind_eq <- go ah bh dep seen
                 body_eq <- go a1_body b1_body (dep+2) seen
                 return $ eras_eq && self_eq && bind_eq && body_eq
               (Lam _ ae _ ab, Lam _ be _ bb) -> do
                 let a1_body = ab (var dep)
                 let b1_body = bb (var dep)
                 let eras_eq = ae == be
                 body_eq <- go a1_body b1_body (dep+1) seen
                 return $ eras_eq && body_eq
               (App _ ae af aa, App _ be bf ba) -> do
                 let eras_eq = ae == be
                 func_eq <- go af bf dep seen
                 argm_eq <- go aa ba dep seen
                 return $ eras_eq && func_eq && argm_eq
               (Let _ _ ax ab, Let _ _ bx bb) -> do
                 let a1_body = ab (var dep)
                 let b1_body = bb (var dep)
                 expr_eq <- go ax bx dep seen
                 body_eq <- go a1_body b1_body (dep+1) seen
                 return $ expr_eq && body_eq
               (Ann _ _ ax _, Ann _ _ bx _) -> go ax bx dep seen
               _ -> return False

data CheckErr = CheckErr Loc Ctx Text deriving Show

check :: Term -> Term -> Module -> Ctx -> Except CheckErr Bool
check trm typ defs ctx = do
  --traceM $ "check"
  --traceM $ show trm
  --traceM $ show typ
  let var n l = Var noLoc $ T.concat [n,"#", T.pack $ show l]
  let typv = reduce typ defs False
  case trm of
    Lam trm_loc trm_eras trm_name trm_body -> case typv of
      All _ typ_eras typ_self typ_name typ_bind typ_body ->
        if typ_eras /= trm_eras 
        then throwError (CheckErr trm_loc ctx "Type mismatch")
        else do
          let self_var = Ann noLoc True trm typ
          let name_var = Ann noLoc True (var trm_name (length ctx + 1)) typ_bind
          let body_typ = typ_body self_var name_var
          let body_ctx = (trm_name,typ_bind):ctx
          check (trm_body name_var) body_typ defs body_ctx
      _  -> do
      --traceM $ show typv
        throwError (CheckErr trm_loc ctx "Lamda has non-function type")
    Let trm_loc trm_name trm_expr trm_body -> do
      expr_typ <- infer trm_expr defs ctx
      let expr_var = Ann noLoc True (var trm_name (length ctx + 1)) expr_typ
      let body_ctx = (trm_name,expr_typ):ctx
      check trm_expr typ defs body_ctx
    _ -> do
      infr <- infer trm defs ctx
      let eq   = equal typ infr defs (length ctx)
      if eq
      then return True
      else throwError (CheckErr noLoc ctx "bad")

infer :: Term -> Module -> Ctx -> Except CheckErr Term
infer trm defs ctx = do
  let var n l = Var noLoc $ T.concat [n,"#", T.pack $ show l]
  case trm of
    Var l n -> return trm
    Ref l n -> case (_defs defs) Map.!? n of
      Just (Expr _ t _) -> return t
      Nothing           -> throwError (CheckErr l ctx (T.concat ["Undefined Reference ", n]))
    Typ l   -> return $ Typ l
    App trm_loc trm_eras trm_func trm_argm -> do
      func_typ <- (\x -> reduce x defs False) <$> infer trm_func defs ctx
      case func_typ of
        All ftyp_loc ftyp_eras ftyp_self_ ftyp_name ftyp_bind ftyp_body -> do
          let self_var = Ann noLoc True trm_func func_typ
          let name_var = Ann noLoc True trm_argm ftyp_bind
          check trm_argm ftyp_bind defs ctx
          let trm_typ = ftyp_body self_var name_var
          if trm_eras /= ftyp_eras
          then throwError $ CheckErr trm_loc ctx "Mismatched Erasure"
          else return trm_typ
        _ -> throwError $ CheckErr trm_loc ctx "Non-function application"
    Let trm_loc trm_name trm_expr trm_body -> do
      expr_typ <- infer trm_expr defs ctx
      let expr_var = Ann noLoc True (var trm_name (length ctx + 1)) expr_typ
      let body_ctx = (trm_name,expr_typ):ctx
      infer (trm_body expr_var) defs body_ctx
    All trm_loc trm_eras trm_self trm_name trm_bind trm_body -> do
      let self_var = Ann noLoc True (var trm_self $ length ctx) trm
      let name_var = Ann noLoc True (var trm_name $ length ctx + 1) trm_bind
      let body_ctx = (trm_name,trm_bind):(trm_self,trm):ctx
      check trm_bind (Typ noLoc) defs ctx
      check (trm_body self_var name_var) (Typ noLoc) defs body_ctx
      return $ Typ noLoc
    Ann trm_loc trm_done trm_expr trm_type -> do
      if trm_done
      then return trm_type
      else do
        check trm_expr trm_type defs ctx
        return trm_type
    _ -> throwError $ CheckErr noLoc ctx "Can't infer type"

checkExpr :: Expr -> Module -> Except CheckErr Bool
checkExpr (Expr n typ trm) mod = do
  --traceM $ "checking: " ++ T.unpack n
  --traceM $ "type: " ++ show typ
  --traceM $ "term: " ++ show trm
  check trm typ mod []

checkModule :: Module -> [Except CheckErr Bool]
checkModule mod = fmap (\(n,x) -> checkExpr x mod) (Map.toList $ _defs mod)

