use p8rs_macros::p8;
use p8rs_piccolo::{FromMultiValue, FromValue, IntoMultiValue, IntoValue, Lua, Table, Value};
use p8rs_types::p8num::P8Num;

#[test]
fn test_conversions() {
    let mut lua = Lua::empty();
    lua.enter(|ctx| {
        let v = (1_i16, true, "hello").into_multi_value(ctx).collect::<Vec<_>>();
        assert!(matches!(
            v.as_slice(),
            [
                Value::Number(P8Num::ONE),
                Value::Boolean(true),
                Value::String(s)
            ] if s == b"hello"
        ));

        let vals = Table::from_value(
            ctx,
            [
                1_i16.into_value(ctx),
                true.into_value(ctx),
                "hello".into_value(ctx),
            ]
            .into_value(ctx),
        )
        .unwrap();

        assert!(matches!(vals.get_value(ctx, p8!(1)), Value::Number(P8Num::ONE)));
        assert!(matches!(vals.get_value(ctx, p8!(2)), Value::Boolean(true)));
        assert!(matches!(vals.get_value(ctx, p8!(3)), Value::String(s) if s == b"hello"));

        let array = <[Value; 3]>::from_value(ctx, vals.into()).unwrap();
        assert!(matches!(
            array.as_slice(),
            [
                Value::Number(P8Num::ONE),
                Value::Boolean(true),
                Value::String(s)
            ] if s == b"hello"
        ));

        let vec = Vec::<Value>::from_value(ctx, vals.into()).unwrap();
        assert!(matches!(
            vec.as_slice(),
            [
                Value::Number(P8Num::ONE),
                Value::Boolean(true),
                Value::String(s)
            ] if s == b"hello"
        ));

        let (a, b, c) = <(i32, bool, String)>::from_multi_value(
            ctx,
            (p8!(2), false, "goodbye").into_multi_value(ctx),
        )
        .unwrap();
        assert_eq!((a, b, c), (2, false, "goodbye".to_owned()));
    });
}

#[test]
fn test_result_conversion() {
    let mut lua = Lua::empty();
    lua.enter(|ctx| {
        let a = Ok::<P8Num, P8Num>(p8!(4)).into_multi_value(ctx).collect::<Vec<_>>();
        assert!(matches!(
            a.as_slice(),
            [Value::Boolean(true), Value::Number(_)]
        ));
        assert_eq!(a[1].to_number().unwrap(), p8!(4));
        
        let b = Err::<P8Num, P8Num>(p8!(7)).into_multi_value(ctx).collect::<Vec<_>>();
        assert!(matches!(
            b.as_slice(),
            [Value::Boolean(false), Value::Number(_)]
        ));
        assert_eq!(b[1].to_number().unwrap(), p8!(7));
        
        let c = Ok::<(P8Num, P8Num, P8Num, P8Num), P8Num>((p8!(1), p8!(2), p8!(3), p8!(4)))
            .into_multi_value(ctx)
            .collect::<Vec<_>>();
        assert!(matches!(
            c.as_slice(),
            [
                Value::Boolean(true),
                Value::Number(_),
                Value::Number(_),
                Value::Number(_),
                Value::Number(_),
            ]
        ));
        assert_eq!(c[1].to_number().unwrap(), p8!(1));
        assert_eq!(c[2].to_number().unwrap(), p8!(2));
        assert_eq!(c[3].to_number().unwrap(), p8!(3));
        assert_eq!(c[4].to_number().unwrap(), p8!(4));
    });
}
