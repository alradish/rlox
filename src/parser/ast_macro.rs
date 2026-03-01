#[macro_export]
macro_rules! lox_ast {
    ($name:ident {
        $(
            $variant:ident(
                $($field:ident: $field_type:ty),* $(,)?
            )
        ),* $(,)?
    }) => {
        paste::paste! {
            #[derive(Debug)]
            pub enum $name {
                $(
                    $variant([<$variant Expr>])
                ),*
            }

            pub trait ExpressionVisitor<R, E> {
                $(
                    fn [<visit_ $variant:snake>](&self, expr: &[<$variant Expr>]) -> Result<R, E>;
                ) *
            }

            impl $name {
                pub fn accept<R, E, T: ExpressionVisitor<R, E>>(&self, visitor: &T) -> Result<R, E> {
                    match self {
                        $(
                            $name::$variant(expr) => expr.accept(visitor),
                        )*
                    }
                }
            }


            $(
                #[derive(Debug)]
                pub struct [<$variant Expr>] {
                    $(pub $field: $field_type),*
                }

                impl From<[<$variant Expr>]> for $name {
                    fn from(expr: [<$variant Expr>]) -> Self {
                        $name::$variant(expr)
                    }
                }

                impl [<$variant Expr>] {
                    pub fn accept<R, E, T: ExpressionVisitor<R, E>>(&self, visitor: &T) -> Result<R, E> {
                        visitor.[<visit_ $variant:snake>](&self)
                    }
                }
            ) *
        }

    };
}
