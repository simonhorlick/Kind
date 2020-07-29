module Core.Rig where

data Rig = Zero | One | Many deriving (Eq, Show)

(+#) :: Rig -> Rig -> Rig
Zero +# x    = x
One  +# Zero = One
One  +# x    = Many
Many +# x    = Many

(*#) :: Rig -> Rig -> Rig
Zero *# x    = Zero
One  *# x    = x
Many *# Zero = Zero
Many *# x    = Many

(≤#) :: Rig -> Rig -> Bool
Zero ≤# x    = True
One  ≤# Zero = False
One  ≤# x    = True
Many ≤# Many = True
Many ≤# x    = False
