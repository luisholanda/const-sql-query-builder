use std::mem::{ManuallyDrop, MaybeUninit};

use crate::const_string::ConstString;

#[const_trait]
pub trait SqlExpression {
    fn write_sql_expression(&self, sql: &mut Sql);

    fn to_sql(&self) -> Sql {
        let mut sql = Sql::default();

        self.write_sql_expression(&mut sql);

        sql
    }
}

impl const SqlExpression for Sql {
    #[inline(always)]
    fn write_sql_expression(&self, sql: &mut Sql) {
        sql.push_str(self.query.as_str());
    }
}

impl<S: ~const SqlExpression, const N: usize> const SqlExpression for [S; N] {
    #[inline(always)]
    fn write_sql_expression(&self, sql: &mut Sql) {
        (self as &[S]).write_sql_expression(sql)
    }
}

impl<S: ~const SqlExpression> const SqlExpression for &[S] {
    fn write_sql_expression(&self, sql: &mut Sql) {
        let mut projs = *self;

        while let Some((cur, rest)) = projs.split_first() {
            cur.write_sql_expression(sql);

            if rest.is_empty() {
                sql.comma();
            }

            projs = rest;
        }
    }
}

macro_rules! impl_sql_expression_tuples {
    (( $($x: ident,)+ )) => {
        impl< $($x),+ > const SqlExpression for ($($x,)+)
        where
            $($x: ~const SqlExpression,)+
        {
            fn write_sql_expression(&self, sql: &mut Sql) {
                let total = ${count(x)};

                $(
                    let val: &$x = &self.${index()};
                    val.write_sql_expression(sql);
                    
                    if ${index()} != total - 1 {
                        sql.comma();
                    }
                )+
            }
        }
    };
}

impl_sql_expression_tuples!((T1,));
impl_sql_expression_tuples!((T1, T2,));
impl_sql_expression_tuples!((T1, T2, T3,));
impl_sql_expression_tuples!((T1, T2, T3, T4,));
impl_sql_expression_tuples!((T1, T2, T3, T4, T5,));
impl_sql_expression_tuples!((T1, T2, T3, T4, T5, T6,));
impl_sql_expression_tuples!((T1, T2, T3, T4, T5, T6, T7,));
impl_sql_expression_tuples!((T1, T2, T3, T4, T5, T6, T7, T8,));
impl_sql_expression_tuples!((T1, T2, T3, T4, T5, T6, T7, T8, T9,));
impl_sql_expression_tuples!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10,));
impl_sql_expression_tuples!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11,));
impl_sql_expression_tuples!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12,));

pub struct Sql {
    query: ConstString,
    bindings: u8,
}

impl const Default for Sql {
    fn default() -> Self {
        Self {
            query: ConstString::default(),
            bindings: 0,
        }
    }
}

impl Sql {
    pub const fn push_str(&mut self, part: &str) -> &mut Sql {
        self.query.push_str(part);
        self
    }

    pub const fn push(&mut self, ch: u8) -> &mut Sql {
        self.query.push_ascii(ch);
        self
    }

    pub const fn push_u64(&mut self, num: u64) -> &mut Sql {
        crate::fmt::fmt_u64(&mut self.query, num);
        self
    }

    pub const fn spacing(&mut self) -> &mut Sql {
        self.push(b' ')
    }

    pub const fn comma(&mut self) -> &mut Sql {
        self.push(b',')
    }

    pub const fn dot(&mut self) -> &mut Sql {
        self.push(b'.')
    }

    pub const fn push_binding(&mut self, binding: &str) -> &mut Sql {
        self.push_str(binding);
        self.bindings += 1;
        self
    }

    pub const fn push_sql(&mut self, other: &Sql) {
        self.query.push_str(other.query.as_str());
        self.bindings += other.bindings;
    }

    pub const fn bindings(&self) -> u8 {
        self.bindings
    }

    pub const fn into_str(self) -> &'static str {
        self.query.leak()
    }

    pub const fn default_array<const N: usize>() -> [Self; N] {
        let mut uninit_array = <MaybeUninit<Sql>>::uninit_array::<N>();

        let mut idx = 0;
        while idx < N {
            uninit_array[idx].write(Sql::default());
            idx += 1;
        }

        let uninit_array = ManuallyDrop::new(uninit_array);

        unsafe {
            std::intrinsics::assert_inhabited::<[Sql; N]>();
            (&uninit_array as *const _ as *const [Sql; N]).read()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concat() {
        const SQL: &str = {
            let mut sql = Sql::default();
            sql.push_str("select").spacing().push_str("bla");
            sql.into_str()
        };

        assert_eq!(SQL, "select bla");
    }
}
