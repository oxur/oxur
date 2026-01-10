;; Test: Impl trait type impl Display
(Item
  :ident (Ident :name "IMPL_TRAIT_TYPE")
  :kind (Const :ty (Ty :kind (ImplTrait ((Trait (PolyTraitRef :trait-ref (TraitRef :path (Path :segments ((PathSegment :ident (Ident :name "Display")))))) None))))))
