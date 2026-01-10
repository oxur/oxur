;; Test: Trait object type without dyn keyword
(Item
  :ident (Ident :name "TRAIT_OBJECT_NO_DYN")
  :kind (Const :ty (Ty :kind (TraitObject
    :bounds ((Trait (PolyTraitRef :trait-ref (TraitRef :path (Path :segments ((PathSegment :ident (Ident :name "Display")))))) None))
    :syntax None))))
