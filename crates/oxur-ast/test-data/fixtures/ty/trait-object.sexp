;; Test: Trait object type dyn Display
(Item
  :ident (Ident :name "TRAIT_OBJECT_TYPE")
  :kind (Const :ty (Ty :kind (TraitObject
    :bounds ((Trait (PolyTraitRef :trait-ref (TraitRef :path (Path :segments ((PathSegment :ident (Ident :name "Display")))))) None))))))
