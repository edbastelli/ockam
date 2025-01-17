use tauri::CustomMenuItem;

use ockam_command::CommandGlobalOpts;

use crate::ctx::TauriCtx;
use crate::tcp::outlet::create::create;
use crate::Result;

pub const TCP_OUTLET_HEADER_MENU_ID: &str = "tcp_outlet_header";
pub const TCP_OUTLET_CREATE_MENU_ID: &str = "tcp_outlet_create";

#[derive(Clone)]
pub struct TcpOutletActions {
    pub options: CommandGlobalOpts,
    pub(crate) menu_items: Vec<CustomMenuItem>,
}

impl TcpOutletActions {
    pub fn new(options: &CommandGlobalOpts) -> TcpOutletActions {
        let header = CustomMenuItem::new(TCP_OUTLET_HEADER_MENU_ID, "TCP Outlets").disabled();
        let create = CustomMenuItem::new(TCP_OUTLET_CREATE_MENU_ID, "Create...");
        let menu_items = vec![header, create];
        TcpOutletActions {
            options: options.clone(),
            menu_items,
        }
    }

    ///
    pub fn full(ctx: &TauriCtx, options: &CommandGlobalOpts) -> Result<TcpOutletActions> {
        let mut s = TcpOutletActions::new(options);
        let mut tcp_outlets = super::list(ctx, options)?
            .list
            .iter()
            .map(|outlet| {
                let outlet_info = format!(
                    "{} to {}",
                    outlet.worker_address().unwrap(),
                    outlet.tcp_addr
                );
                CustomMenuItem::new(outlet_info.clone(), outlet_info)
            })
            .collect::<Vec<CustomMenuItem>>();
        s.menu_items.append(&mut tcp_outlets);
        Ok(s)
    }
}

/// Event listener for the "Create..." menu item
pub fn on_create(ctx: TauriCtx, options: &CommandGlobalOpts) -> tauri::Result<()> {
    let _ = create(ctx, options);
    Ok(())
}
