module Core where

import           System.Exit
import           Text.Megaparsec
import           Data.Text                  (Text)
import qualified Data.Text                  as T
import qualified Data.Text.IO               as TIO
import Control.Monad.Except

import           Core.Rig
import           Core.Eval
import           Core.Hash
import qualified Core.Parse      as Parse
import qualified Core.Print      as Print
import           Core.Type
import           Core.Check

parseTerm :: Text -> IO Term
parseTerm txt = case parse Parse.term "" txt of
  Left  e -> putStr (errorBundlePretty e) >> exitFailure
  Right t -> return (t $ [])

parseFile :: FilePath -> IO Module
parseFile file = do
  txt <- TIO.readFile file
  case parse Parse.modl file txt of
    Left  e -> putStr (errorBundlePretty e) >> exitFailure
    Right m -> return m

