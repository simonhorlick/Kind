module Main where

import Core
import System.Directory
import System.FilePath
import Control.Monad

main :: IO ()
main = do
  files <- getCurrentDirectory
             >>= getDirectoryContents
             >>= filterM doesFileExist
             >>= filterM (pure . isExtensionOf ".fmc")
  traverse (\f -> putStrLn f >> parseFile f) files
  return ()
