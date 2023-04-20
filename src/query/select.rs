use std::marker::Destruct;

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

        self.projections.write_sql_expression(sql);

        sql.push_str(" FROM ");
        self.from.write_sql_expression(sql);

        if let Some(offset) = self.offset {
            sql.push_str(" OFFSET ").push_u64(offset);
        }

        if let Some(limit) = self.limit {
            sql.push_str(" LIMIT ").push_u64(limit);
        }
    }
}

macro_rules! impl_join {
    ($method: ident, $style: ident) => {
        pub const fn $method<Rhs>(self, rhs: Rhs) -> IncompleteSelectJoin<Source, Rhs, N> {
            IncompleteSelectJoin {
                select: self,
                right: rhs,
                style: JoinStyle::$style
            }
        }
    };
}

impl<Source, const N: usize> Select<Source, N> {
    pub const fn select<P>(self, projection: P) -> Select<Source, { P::LENGTH }>
    where
        P: ~const AsProjection + ~const Destruct,
        Source: ~const Destruct,
    {
        Select {
            from: self.from,
            projections: projection.as_projection(),
            limit: self.limit,
            offset: self.offset,
        }
    }

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

pub struct Join<Lhs, Rhs, On> {
    left: Lhs,
    right: Rhs,
    style: JoinStyle,
    on: JoinOn<On>,
}

pub enum JoinOn<On> {
    Explicit(On),
    Using(On),
    Natural,
}

impl<Lhs, Rhs, On> const SqlExpression for Join<Lhs, Rhs, On>
where
    Lhs: ~const SqlExpression,
    Rhs: ~const SqlExpression,
    On: ~const SqlExpression,
{
    fn write_sql_expression(&self, sql: &mut Sql) {
        self.left.write_sql_expression(sql);

        if matches!(self.on, JoinOn::Natural) {
            sql.push_str(" NATURAL");
        }

        match self.style {
            JoinStyle::Inner => sql.push_str(" INNER JOIN "),
            JoinStyle::Left => sql.push_str(" LEFT OUTER JOIN "),
            JoinStyle::Cross => sql.push_str(" CROSS JOIN "),
        };

        self.right.write_sql_expression(sql);

        match &self.on {
            JoinOn::Explicit(on) => on.write_sql_expression(sql.push_str(" ON ")),
            JoinOn::Using(columns) => columns.write_sql_expression(sql.push_str(" USING ")),
            JoinOn::Natural => {},
        }
    }
}

pub struct IncompleteSelectJoin<Lhs, Rhs, const N: usize> {
    select: Select<Lhs, N>,
    right: Rhs,
    style: JoinStyle,
}

impl<Lhs, Rhs, const N: usize> IncompleteSelectJoin<Lhs, Rhs, N> {
    pub fn on<On>(self, on: On) -> Select<Join<Lhs, Rhs, On>, N> {
        self.construct(JoinOn::Explicit(on))
    }

    pub fn using<On>(self, columns: On) -> Select<Join<Lhs, Rhs, On>, N> {
        self.construct(JoinOn::Using(columns))
    }

    pub fn natural(self) -> Select<Join<Lhs, Rhs, ()>, N> {
        self.construct(JoinOn::Natural)
    }

    fn construct<On>(self, join_on: JoinOn<On>) -> Select<Join<Lhs, Rhs, On>, N> {
        Select {
            from: Join {
                left: self.select.from,
                right: self.right,
                style: self.style,
                on: join_on,
            },
            projections: self.select.projections,
            offset: self.select.offset,
            limit: self.select.limit
        }
    }
}
