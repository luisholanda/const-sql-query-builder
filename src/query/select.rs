use std::marker::Destruct;

use crate::{
    expression::{Sql, SqlExpression},
    schema::{table_columns, Table, Column},
};

pub const fn from(table: Table) -> Select<Table, &'static [Column]> {
    Select {
        from: table,
        projections: table_columns(table),
        limit: None,
        offset: None,
    }
}

pub struct Select<Source, Proj> {
    from: Source,
    projections: Proj,
    limit: Option<u64>,
    offset: Option<u64>,
}

impl<Source, Proj> const SqlExpression for Select<Source, Proj>
where
    Source: ~const SqlExpression,
    Proj: ~const SqlExpression,
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
        pub const fn $method<Rhs>(self, rhs: Rhs) -> IncompleteSelectJoin<Source, Rhs, Proj> {
            IncompleteSelectJoin {
                select: self,
                right: rhs,
                style: JoinStyle::$style
            }
        }
    };
}

impl<Source, Proj> Select<Source, Proj> {
    pub const fn select<P>(self, projections: P) -> Select<Source, P>
    where
        Self: ~const Destruct,
    {
        Select {
            from: self.from,
            projections,
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

pub struct IncompleteSelectJoin<Lhs, Rhs, Proj> {
    select: Select<Lhs, Proj>,
    right: Rhs,
    style: JoinStyle,
}

impl<Lhs, Rhs, Proj> IncompleteSelectJoin<Lhs, Rhs, Proj> {
    pub fn on<On>(self, on: On) -> Select<Join<Lhs, Rhs, On>, Proj> {
        self.construct(JoinOn::Explicit(on))
    }

    pub fn using<On>(self, columns: On) -> Select<Join<Lhs, Rhs, On>, Proj> {
        self.construct(JoinOn::Using(columns))
    }

    pub fn natural(self) -> Select<Join<Lhs, Rhs, ()>, Proj> {
        self.construct(JoinOn::Natural)
    }

    fn construct<On>(self, join_on: JoinOn<On>) -> Select<Join<Lhs, Rhs, On>, Proj> {
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
