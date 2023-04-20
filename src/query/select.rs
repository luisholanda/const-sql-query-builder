use crate::{
    expression::{Sql, SqlExpression},
    schema::{table_columns, Table},
};

pub const fn from<const TABLE: Table>() -> Select<Table, { table_columns(TABLE).len() }> {
    let mut projections = Sql::default_array::<{ table_columns(TABLE).len() }>();

    let columns = table_columns(TABLE);
    let mut idx = 0;
    while idx < columns.len() {
        columns[idx].write_sql_expression(&mut projections[idx]);
        idx += 1;
    }

    Select {
        from: TABLE,
        projections,
        limit: None,
        offset: None,
    }
}

pub struct Select<Source, const N: usize> {
    from: Source,
    // TODO: this should be moved to the type level, so that we don't need to concat multiple Sql
    // instances and have a `N` const parameter.
    projections: [Sql; N],
    limit: Option<u64>,
    offset: Option<u64>,
}

impl<Source, const N: usize> const SqlExpression for Select<Source, N>
where
    Source: ~const SqlExpression,
{
    fn write_sql_expression(&self, sql: &mut Sql) {
        sql.push_str("SELECT ");

        let mut projs = &self.projections as &[Sql];
        while let Some((cur, rest)) = projs.split_first() {
            sql.push_sql(cur);

            if rest.is_empty() {
                sql.comma();
            }

            projs = rest;
        }

        sql.push_str(" FROM ");
        self.from.write_sql_expression(sql);

        // TODO: find a way to write u64 as string in a const context.
    }
}

macro_rules! impl_join {
    ($method: ident, $style: ident) => {
        pub const fn $method<Rhs>(
            self,
            rhs: Rhs,
        ) -> Select<Join<Source, Rhs, { JoinStyle::$style }>, N> {
            Select {
                from: Join {
                    left: self.from,
                    right: rhs,
                },
                projections: self.projections,
                limit: self.limit,
                offset: self.offset,
            }
        }
    };
}

impl<Source, const N: usize> Select<Source, N> {
    // TODO: need destructors.
    //pub const fn select<P>(self, projection: P) -> Select<Source, { P::LENGTH }>
    //where
    //    P: ~const AsProjection,
    //{
    //    Select {
    //        from: self.from,
    //        projections: projection.as_projection(),
    //        limit: self.limit,
    //    }
    //}

    impl_join!(inner_join, Inner);
    impl_join!(left_join, Left);
    impl_join!(cross_join, Cross);

    pub const fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub const fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }
}

#[const_trait]
pub trait AsProjection {
    const LENGTH: usize;

    fn as_projection(&self) -> [Sql; Self::LENGTH];
}

macro_rules! impl_as_projection_tuples {
    (( $($x: ident,)+ )) => {
        impl< $($x),+ > const AsProjection for ($($x,)+)
        where
            $($x: ~const SqlExpression,)+
        {
            const LENGTH: usize = ${count(x)};

            fn as_projection(&self) -> [Sql; Self::LENGTH] {
                let mut projections = Sql::default_array::<{Self::LENGTH}>();

                $(
                let val: &$x = &self.${index()};
                val.write_sql_expression(&mut projections[${index()}]);
                )+

                projections
            }
        }
    };
}

impl_as_projection_tuples!((T1,));
impl_as_projection_tuples!((T1, T2,));
impl_as_projection_tuples!((T1, T2, T3,));
impl_as_projection_tuples!((T1, T2, T3, T4,));
impl_as_projection_tuples!((T1, T2, T3, T4, T5,));
impl_as_projection_tuples!((T1, T2, T3, T4, T5, T6,));
impl_as_projection_tuples!((T1, T2, T3, T4, T5, T6, T7,));
impl_as_projection_tuples!((T1, T2, T3, T4, T5, T6, T7, T8,));
impl_as_projection_tuples!((T1, T2, T3, T4, T5, T6, T7, T8, T9,));
impl_as_projection_tuples!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10,));
impl_as_projection_tuples!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11,));
impl_as_projection_tuples!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12,));

#[derive(PartialEq, Eq)]
pub enum JoinStyle {
    Inner,
    Left,
    Cross,
}

pub struct Join<Lhs, Rhs, const STYLE: JoinStyle> {
    left: Lhs,
    right: Rhs,
}

impl<Lhs, Rhs, const STYLE: JoinStyle> const SqlExpression for Join<Lhs, Rhs, STYLE>
where
    Lhs: ~const SqlExpression,
    Rhs: ~const SqlExpression,
{
    fn write_sql_expression(&self, sql: &mut Sql) {
        self.left.write_sql_expression(sql);

        match STYLE {
            JoinStyle::Inner => sql.push_str(" INNER JOIN "),
            JoinStyle::Left => sql.push_str(" LEFT OUTER JOIN "),
            JoinStyle::Cross => sql.push_str(" CROSS JOIN "),
        };

        self.right.write_sql_expression(sql);
    }
}
