use core::cmp::Ordering;
use p8rs_macros::p8;
use p8rs_piccolo::{Lua, Table, Value};

#[test]
fn test_table_iter() {
    let mut lua = Lua::core();

    lua.enter(|ctx| {
        let table = Table::new(&ctx);

        table.set(ctx, p8!(1), "1").unwrap();
        table.set(ctx, p8!(2), "2").unwrap();
        table.set(ctx, p8!(3), "3").unwrap();
        table.set(ctx, "1", p8!(1)).unwrap();
        table.set(ctx, "2", p8!(2)).unwrap();
        table.set(ctx, "3", p8!(3)).unwrap();

        let mut pairs = table.iter().collect::<Vec<_>>();
        pairs.sort_by(|&(ak, _), &(bk, _)| match (ak, bk) {
            (Value::Number(a), Value::Number(b)) => a.cmp(&b),
            (Value::Number(_), Value::String(_)) => Ordering::Less,
            (Value::String(_), Value::Number(_)) => Ordering::Greater,
            (Value::String(a), Value::String(b)) => a.cmp(&b),
            _ => unreachable!(),
        });

        assert_eq!(pairs.len(), 6);
        assert!(matches!(pairs[0], (Value::Number(n), Value::String(s)) if s == "1" && i16::from(n) == 1 ));
        assert!(matches!(pairs[1], (Value::Number(n), Value::String(s)) if s == "2" && i16::from(n) == 2 ));
        assert!(matches!(pairs[2], (Value::Number(n), Value::String(s)) if s == "3" && i16::from(n) == 3 ));
        assert!(matches!(pairs[3], (Value::String(s), Value::Number(n)) if s == "1" && i16::from(n) == 1 ));
        assert!(matches!(pairs[4], (Value::String(s), Value::Number(n)) if s == "2" && i16::from(n) == 2 ));
        assert!(matches!(pairs[5], (Value::String(s), Value::Number(n)) if s == "3" && i16::from(n) == 3 ));

        for (k, _) in table.iter() {
            table.set(ctx, k, Value::Nil).unwrap();
        }

        assert!(table.get_value(ctx, p8!(1)).is_nil());
        assert!(table.get_value(ctx, p8!(2)).is_nil());
        assert!(table.get_value(ctx, p8!(3)).is_nil());
        assert!(table.get_value(ctx, "1").is_nil());
        assert!(table.get_value(ctx, "2").is_nil());
        assert!(table.get_value(ctx, "3").is_nil());
    });
}
