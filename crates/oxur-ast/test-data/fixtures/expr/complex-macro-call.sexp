(Expr
  :id 100
  :kind (MacCall
          (MacCall
            :path (Path
                    :segments ((PathSegment :ident (Ident :name "println") :id 101))
                    :span (Span :lo 0 :hi 7))
            :args (Delimited
                    :dspan (DelSpan :open (Span :lo 7 :hi 8) :close (Span :lo 20 :hi 21))
                    :delim Paren
                    :tokens (TokenStream :source "\"Hello, world!\""))
            :prior-type-ascription nil))
  :span (Span :lo 0 :hi 21)
  :attrs ())
