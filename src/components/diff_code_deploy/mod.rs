use color_eyre::eyre::ContextCompat;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = DiffCodeDeployContext)]
pub struct DiffCodeDeploy {
    /// On which account do you want to compare local components?
    account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: near_cli_rs::network::Network,
}

#[derive(Clone)]
pub struct DiffCodeDeployContext(near_cli_rs::network::NetworkContext);

impl DiffCodeDeployContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<DiffCodeDeploy as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_network_callback: near_cli_rs::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let account_id: near_primitives::types::AccountId = scope.account_id.clone().into();

                move |network_config| {
                    let near_social_account_id = crate::consts::NEAR_SOCIAL_ACCOUNT_ID
                        .get(network_config.network_name.as_str())
                        .wrap_err_with(|| {
                            format!(
                                "The <{}> network does not have a near-social contract.",
                                network_config.network_name
                            )
                        })?;

                    let local_components = crate::common::get_local_components()?;
                    if local_components.is_empty() {
                        println!("There are no components in the current ./src folder. Goodbye.");
                        return Ok(());
                    }

                    let remote_components = crate::common::get_remote_components(
                        network_config,
                        &local_components,
                        near_social_account_id,
                        &account_id,
                    )?;

                    if !remote_components.is_empty() {
                        let updated_components = crate::common::get_updated_components(
                            local_components,
                            &remote_components,
                        );
                        if updated_components.is_empty() {
                            println!("There are no new or modified components in the current ./src folder. Goodbye.");
                            return Ok(());
                        }
                    } else {
                        println!("\nAll local components are new to <{account_id}>.");
                    };
                    Ok(())
                }
            });
        Ok(Self(near_cli_rs::network::NetworkContext {
            config: previous_context.config,
            on_after_getting_network_callback,
        }))
    }
}

impl From<DiffCodeDeployContext> for near_cli_rs::network::NetworkContext {
    fn from(item: DiffCodeDeployContext) -> Self {
        item.0
    }
}
