;; Test: Trait object with invalid syntax (should fail)
(Item
  :ident (Ident :name "INVALID_SYNTAX")
  :kind (Const :ty (Ty :kind (TraitObject
    :bounds ()
    :syntax Invalid))))
