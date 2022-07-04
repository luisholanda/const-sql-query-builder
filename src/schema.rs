use crate::expression::{Sql, SqlExpression};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Ident {
    pub name: &'static str,
    pub schema: &'static str,
}

impl const SqlExpression for Ident {
    fn write_sql_expression(&self, sql: &mut Sql) {
        sql.push_str(self.schema).dot().push_str(self.name);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Column {
    pub name: &'static str,
    pub table: &'static Table,
}

impl const SqlExpression for Column {
    fn write_sql_expression(&self, sql: &mut Sql) {
        sql.push_str(self.table.ident.name)
            .dot()
            .push_str(self.name);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Table {
    pub ident: Ident,
    pub all_columns: &'static [Column],
}

pub const fn table_columns(table: Table) -> &'static [Column] {
    table.all_columns
}

impl const SqlExpression for Table {
    fn write_sql_expression(&self, sql: &mut Sql) {
        self.ident.write_sql_expression(sql);
    }
}
