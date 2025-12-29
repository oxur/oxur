(Expr
  :id 1
  :kind (MacCall
          (MacCall
            :path (Path :segments ((PathSegment :ident (Ident :name "vec"))))
            :args (Delimited
                    :dspan (DelSpan :open (Span :lo 0 :hi 0) :close (Span :lo 0 :hi 0))
                    :delim Bracket
                    :tokens (TokenStream :source "1, 2, 3"))
            :prior-type-ascription nil))
  :span (Span :lo 0 :hi 0)
  :attrs ())
