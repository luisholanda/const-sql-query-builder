use crate::const_string::ConstString;

pub trait SqlExpression {
    fn write_sql_expression(&self, sql: &mut Sql);
}

#[derive(Debug, Clone, Copy)]
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
        self.concat(binding);
        self.bindings += 1;
        self
    }

    pub const fn bindings(&self) -> u8 {
        self.bindings
    }

    pub const fn into_str(self) -> &'static str {
        self.query.leak()
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
