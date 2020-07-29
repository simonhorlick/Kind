{-# LANGUAGE DerivingVia #-}
{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE DataKinds #-}
module Core.Hash where

import           Data.Bits (Bits, shiftL, shiftR, xor, (.&.))
import           Data.Text (Text)
import qualified Data.Text as T hiding (find)
import qualified Data.Text.Read as T
import qualified Data.Text.Encoding as T
import           Data.Word
import           Data.Char

import           Core.Rig
import           Core.Type
import           Core.Print

import qualified BLAKE3 as Blake
import qualified Data.ByteArray as BA

import qualified Data.ByteString as BS
import Data.ByteString (ByteString)
import Data.ByteString.Internal (unpackChars)
import qualified Data.ByteString.Base16 as BS16
import qualified Data.ByteString.Base16 as BS16

newtype Hash = Hash {_bytes :: ByteString} deriving (Eq,Ord) via ByteString

instance Show Hash where
  show (Hash bytes) = unpackChars $ (BS.append "0x" $ BS16.encode bytes)

-- simple MurmurHash implementation
--mix64 :: Word64 -> Word64
--mix64 h =
--  let h1     = xor h (shiftR h 33)
--      h2     = h1 * 0xff51afd7ed558ccd
--      h3     = xor h2 (shiftR h2 33)
--      h4     = h3 * 0xc4ceb9fe1a85ec53
--   in xor h4 (shiftR h4 33)
--
--hashTwo :: Hash -> Hash -> Hash
--hashTwo x y = x .&. (shiftL y 32)
--
--instance Semigroup Hash where
--  (<>) = hashTwo
--
--instance Monoid Hash where
--  mempty = 0
--  mappend = (<>)
--
--hashText :: Text -> Hash
--hashText txt = T.foldr (\c h -> (fromIntegral $ ord c) <> h) 0 txt

-- TODO: optimize with incremental hasher
blake :: Term -> Blake.Digest 32
blake term = Blake.hash $ [T.encodeUtf8 $ compText term 0]

hash :: Term -> Hash
hash term = Hash $ BA.convert $ blake term

compText :: Term -> Integer -> Text
compText term dep =
  let go = compText
      var = Var noLoc
      cons c s = T.cons c (T.pack $ show s)
   in case term of
  Var _ _ idx          ->
    if idx < 0
    then T.concat ["^", T.pack $ show $ dep + idx]
    else T.concat ["#", T.pack $ show idx]
  Ref _ n              -> T.concat ["&", n]
  Typ _                -> "*"
  All _ r _ _ h b ->
    let bind = go h dep
        s    = (var "" (0-dep-1))
        x    = (var "" (0-dep-2))
        body = go (b s x) (dep + 2)
        q    = case r of {Zero -> "0"; One -> "1"; Many -> "ω";}
     in T.concat [q,"Π",bind,body]
  Lam _ n b          ->
    let body = go (b (var "" (0-dep-1))) (dep+1)
    in T.concat ["λ", body]
  App _ f a      -> T.concat ["@", go f dep, go a dep]
  Let _ _ x b        ->
    let expr = go x dep
        body = go (b (var "" (0-dep-1))) (dep+1)
     in T.concat ["$",expr,body]
  Ann _ _ x _          -> go x dep
