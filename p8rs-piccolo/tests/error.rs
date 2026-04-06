use p8rs_piccolo::{error::LuaError, Callback, Closure, Error, Executor, ExternError, Lua, Value};
use thiserror::Error;

#[test]
#[ignore]
fn error_unwind() -> Result<(), ExternError> {
    let mut lua = Lua::empty();

    let executor = lua.try_enter(|ctx| {
        let closure = Closure::load(
            ctx,
            None,
            &br#"
                function do_error()
                    error('test error')
                end

                do_error()
            "#[..],
        )?;
        Ok(ctx.stash(Executor::start(ctx, closure.into(), ())))
    })?;

    lua.finish(&executor, &mut ()).unwrap();
    lua.try_enter(|ctx| {
        match ctx.fetch(&executor).take_result::<()>(ctx)? {
            Err(Error::Lua(LuaError(Value::String(s)))) => assert!(s == "test error"),
            _ => panic!("wrong error returned"),
        }
        Ok(())
    })
}

#[test]
#[ignore]
fn error_tostring() -> Result<(), ExternError> {
    let mut lua = Lua::empty();

    #[derive(Debug, Error)]
    #[error("test error")]
    struct TestError;

    let executor = lua.try_enter(|ctx| {
        let callback = Callback::from_fn(&ctx, |_, _, _, _| Err(TestError.into()));
        ctx.set_global(b"callback", callback);

        let closure = Closure::load(
            ctx,
            None,
            &br#"
                local r, e = pcall(callback)
                assert(not r)
                assert(tostring(e) == "test error")
            "#[..],
        )?;

        Ok(ctx.stash(Executor::start(ctx, closure.into(), ())))
    })?;

    lua.execute(&executor, &mut ())
}
