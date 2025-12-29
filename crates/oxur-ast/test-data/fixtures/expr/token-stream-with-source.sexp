(Expr
  :id 1
  :kind (MacCall
          (MacCall
            :path (Path :segments ((PathSegment :ident (Ident :name "test"))))
            :args (Delimited
                    :dspan (DelSpan :open (Span :lo 0 :hi 0) :close (Span :lo 0 :hi 0))
                    :delim Paren
                    :tokens (TokenStream :source "hello world"))
            :prior-type-ascription nil))
  :span (Span :lo 0 :hi 0)
  :attrs ())
