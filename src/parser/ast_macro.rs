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
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
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

                $(
                    pub fn [<$variant:snake>]($($field: $field_type),*) -> $name {
                        $name::$variant([<$variant Expr>]::new($($field),*))
                    }
                ) *
            }


            $(
                #[derive(Debug, serde::Serialize, serde::Deserialize)]
                pub struct [<$variant Expr>] {
                    $(pub $field: $field_type),*
                }

                impl From<[<$variant Expr>]> for $name {
                    fn from(expr: [<$variant Expr>]) -> Self {
                        $name::$variant(expr)
                    }
                }

                impl [<$variant Expr>] {
                    pub fn new($($field: $field_type),*) -> Self {
                        Self { $($field),* }
                    }

                    pub fn accept<R, E, T: ExpressionVisitor<R, E>>(&self, visitor: &T) -> Result<R, E> {
                        visitor.[<visit_ $variant:snake>](&self)
                    }
                }
            ) *
        }

    };
}
