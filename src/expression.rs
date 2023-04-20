use std::mem::{ManuallyDrop, MaybeUninit};

use crate::const_string::ConstString;

#[const_trait]
pub trait SqlExpression {
    fn write_sql_expression(&self, sql: &mut Sql);
}

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
