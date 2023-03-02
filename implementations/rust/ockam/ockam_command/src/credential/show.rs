use clap::{arg, Args};
use colorful::Colorful;
use ockam::Context;
use ockam_identity::IdentityIdentifier;

use crate::{
    credential::validate_encoded_cred, util::node_rpc, vault::default_vault_name, CommandGlobalOpts,
};

#[derive(Clone, Debug, Args)]
pub struct ShowCommand {
    #[arg()]
    pub credential_name: String,

    #[arg(default_value_t = default_vault_name())]
    pub vault: String,
}

impl ShowCommand {
    pub fn run(self, opts: CommandGlobalOpts) {
        node_rpc(run_impl, (opts, self));
    }
}

async fn run_impl(
    ctx: Context,
    (opts, cmd): (CommandGlobalOpts, ShowCommand),
) -> crate::Result<()> {
    display_credential(&opts, &ctx, &cmd.credential_name, &cmd.vault).await?;

    Ok(())
}

pub(crate) async fn display_credential(
    opts: &CommandGlobalOpts,
    ctx: &Context,
    cred_name: &str,
    vault_name: &str,
) -> crate::Result<()> {
    let cred_config = opts.state.credentials.get(&cred_name)?.config().await?;

    let issuer = IdentityIdentifier::try_from(cred_config.issuer.to_string())?;
    let is_verified = match validate_encoded_cred(
        &cred_config.encoded_credential,
        &issuer,
        &vault_name,
        &opts,
        &ctx,
    )
    .await
    {
        Ok(_) => "✔︎".light_green(),
        Err(_) => "✕".light_red(),
    };

    let cred = cred_config.credential()?;
    println!("Credential: {cred_name} {is_verified}");
    println!("{cred}");

    Ok(())
}