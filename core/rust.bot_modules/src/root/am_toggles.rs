use futures_util::FutureExt;

pub async fn setup(data: &base_data::Data) -> Result<(), base_data::Error> {
    let props = data.props.clone();
    data.props.add_permodule_function(
        "root",
        "reset_can_use_bot_whitelist",
        Box::new(move |_, _| {
            {
                let props = props.clone();
                async move { reset_can_use_bot_whitelist(&*props).await }.boxed()
                // Work around rust lifetime issue
            }
            .boxed()
        }),
    );

    Ok(())
}

/// No arguments required
pub async fn reset_can_use_bot_whitelist(
    props: &dyn base_data::Props,
) -> Result<(), base_data::Error> {
    props.reset_can_use_bot().await
}
