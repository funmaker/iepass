use p8rs_piccolo::{Callback, CallbackReturn, Executor, ExternError, Function, Lua, Variadic};

#[test]
fn function_compose_bind() -> Result<(), ExternError> {
    let mut lua = Lua::core();

    let executor = lua.try_enter(|ctx| {
        let composed_functions = Function::compose(
            &ctx,
            [
                Callback::from_fn(&ctx, |ctx, _, mut stack| {
                    let i: Variadic<Vec<i16>> = stack.consume(ctx)?;
                    stack.replace(ctx, i.into_iter().sum::<i16>());
                    Ok(CallbackReturn::Return)
                })
                .into(),
                Callback::from_fn(&ctx, |ctx, _, mut stack| {
                    let i: i16 = stack.consume(ctx)?;
                    stack.replace(ctx, i * 2);
                    Ok(CallbackReturn::Return)
                })
                .into(),
                Callback::from_fn(&ctx, |ctx, _, mut stack| {
                    let i: i16 = stack.consume(ctx)?;
                    stack.replace(ctx, i + 1);
                    Ok(CallbackReturn::Return)
                })
                .into(),
                Callback::from_fn(&ctx, |ctx, _, mut stack| {
                    let i: i16 = stack.consume(ctx)?;
                    stack.replace(ctx, i * 3);
                    Ok(CallbackReturn::Return)
                })
                .into(),
            ],
        )
        .bind(&ctx, 1_i16)
        .bind(&ctx, (2_i16, 1_i16));
        Ok(ctx.stash(Executor::start(ctx, composed_functions, 1_i16)))
    })?;

    assert_eq!(lua.execute::<i16>(&executor)?, 33);
    Ok(())
}
