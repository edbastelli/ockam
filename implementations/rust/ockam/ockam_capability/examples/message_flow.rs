#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
extern crate tracing;

use core::time::Duration;

use ockam::identity::{Identity, TrustEveryonePolicy};
use ockam::vault::Vault;
use ockam::{route, Address, Context, Message, Result, Routed, TcpTransport, Worker, TCP};

/// ## How will Connor’s Ockam Command ensure message flow is:
///
///     transport < - > secure channel < - > app workers?
///
///
/// ## Suborbital use case
///
/// ### Connor's Ockam Command does:
///
///   #1 create an identity for their control plane
///
///   #2 create an identity for their edge plane
///
///   #3 provision their edge plane with the identity of their control plane
///
///   #4 provision their control plane with the identity of their edge plane
///
///   #5 start a httpd on their control plane with:
///
///          bind_addr: 127.0.0.1:4001
///
///   #6 create an outlet on ockamcloud to their control plane with:
///
///         cloud_addr: suborbital.node.ockam.network:4000
///              alias: mrinal_cp
///      outlet_target: host.docker.internal:4001 (httpd started in #1)
///
///   #7 create an inlet on ockamcloud from their edge plane with:
///
///         cloud_addr: suborbital.node.ockam.network:4000
///              alias: mrinal_cp
///      inlet_address: 127.0.0.1:4002
///
///   #8 send a http request from their edge plane to the httpd on their control plane with:
///
///           protocol: http
///          authority: 127.0.0.1:4002
///             method: GET
///               path: /
///
/// ### Connor’s Ockam Command needs:
///
///   - An address for Ockam Cloud
///   - AuthN to establish their identity with Ockam Cloud
///   - AuthZ to send messages to Ockam Cloud
///   - AuthZ to create an outlet on Ockam Cloud
///   - AuthZ to create an inlet on Ockam Cloud
///
/// ### Connor’s Outlet needs:
///
///   - AuthN to establish their identity with Connor’s Inlet
///   - AuthZ to send messages to Connor’s Inlet
///
/// ### Connor’s Inlet needs:
///
///   - AuthN to establish their identity with Connor’s Outlet
///   - AuthZ to send messages to Connor’s Outlet
///
/// ### Connor’s httpd needs:
///
///   - AuthZ to receive requests from Connor’s Outlet
///
/// ### Connor’s curl needs:
///
///   - AuthZ to connect to Connor’s Inlet
///
/// ### Connor’s Workers need:
///
///   - TcpTransport needs AuthZ to send messages to Worker:
///       connectivity by parenthood: Worker -> TcpTransport(Worker)
///
///   - SecureChannel needs AuthZ to send messages to TcpTransport:
///       connectivity by endowment: Worker -> SecureChannel(TcpTransport)
// - ContextAuthorization -----------------------------------------------------
use ockam_core::{async_trait, AccessControl, Action, LocalMessage};

struct ContextAuthorization(&'static str);

#[ockam_core::async_trait]
impl AccessControl for ContextAuthorization {
    async fn is_authorized(&self, local_msg: &LocalMessage) -> Result<bool> {
        warn!(
            "ContextAuthorization::{}::is_authorized -> {}",
            self.0,
            local_msg.transport()
        );
        let payload = &local_msg.transport().payload[1..];
        warn!("\tPayload: {:?}", String::from_utf8_lossy(payload));
        ockam_core::allow()
    }
}

// - attribute based access control (abac) ------------------------------------

use ockam_capability::abac::{mem::Memory as AbacBackend, Abac};

fn default_abac() -> AbacBackend {
    AbacBackend::new()
}

// - ockam::node --------------------------------------------------------------

#[ockam::node]
async fn main(mut ctx: Context) -> ockam::Result<()> {
    // start up app worker
    let _join_handle: std::thread::JoinHandle<Result<(), ockam::Error>> =
        std::thread::spawn(|| {
            let (mut ctx, mut executor) = ockam::start_node();
            let _ = executor.execute(async move {
                ctx.set_access_control(ContextAuthorization("appworker"))
                    .await?;
                appworker_main_tcp(ctx).await
            })?;
            Ok(())
        });
    //std::thread::sleep(Duration::from_secs(1));

    ctx.set_access_control(ContextAuthorization("connor"))
        .await?;

    //match connor_main_secure_channel(ctx).await {
    match connor_main_tcp(ctx).await {
        //match connor_main_app(ctx).await {
        Ok(_) => (),
        Err(e) => {
            error!("connor_main exited with error: {}", e);
        }
    }

    Ok(())
}

// - Connor -------------------------------------------------------------------

async fn connor_main_tcp(mut ctx: Context) -> ockam::Result<()> {
    // Initialize the TCP Transport.
    let mut tcp = TcpTransport::create(&ctx).await?;

    // provision the TCP transport with the capability to communicate with connor's app
    let capability = ctx.cap().request(Action::SendMessage, "app").await?;
    tcp.cap().provision(capability, "app")?;

    // Send a message to the "appworker" worker, on a different node, over a tcp transport.
    let r = route![(TCP, "localhost:4000"), "appworker"];
    ctx.send(r, "Hello Ockam!".to_string()).await?;

    // Wait to receive a reply and print it.
    let reply = ctx.receive::<String>().await?;
    info!("App Received: {}", reply); // should print "Hello Ockam!"

    Ok(())
}

async fn connor_main_secure_channel(mut ctx: Context) -> ockam::Result<()> {
    let vault = Vault::create();
    let bob = Identity::create(&ctx, &vault).await?;

    // Create a secure channel listener for Bob that will wait for requests to
    // initiate an Authenticated Key Exchange.
    bob.create_secure_channel_listener("bob", TrustEveryonePolicy)
        .await?;

    // Create an Identity to represent Connor.
    let connor = Identity::create(&ctx, &vault).await?;

    // As Connor, connect to Bob's secure channel listener and perform an
    // Authenticated Key Exchange to establish an encrypted secure channel with Bob.
    let channel = connor
        .create_secure_channel(route!["bob"], TrustEveryonePolicy)
        .await?;

    ctx.send(route![channel, "app"], "GET /some/api".to_string())
        .await?;

    let message = ctx.receive::<String>().await?;
    info!("App Received: {}", message);

    ctx.stop().await
}

async fn connor_main_app(mut ctx: Context) -> ockam::Result<()> {
    ctx.send(route!["app"], "Hello Ockam!".to_string()).await?;

    let message = ctx.receive::<String>().await?;
    info!("Received: {}", message);

    ctx.stop().await
}

// - AppWorker ----------------------------------------------------------------

async fn appworker_main_app(mut ctx: Context) -> ockam::Result<()> {
    let message = ctx.receive::<String>().await?;
    info!("App Received: {}", message);

    // Don't call ctx.stop() here so this node runs forever.
    Ok(())
}

async fn appworker_main_tcp(ctx: Context) -> ockam::Result<()> {
    // Initialize the TCP Transport.
    let tcp = TcpTransport::create(&ctx).await?;

    // Create a TCP listener and wait for incoming connections.
    tcp.listen("127.0.0.1:4000").await?;

    // Create an AppWorker worker
    ctx.start_worker("appworker", AppWorker).await?;

    // Don't call ctx.stop() here so this node runs forever.
    Ok(())
}

struct AppWorker;

#[ockam::worker]
impl Worker for AppWorker {
    type Context = Context;
    type Message = String;

    async fn initialize(&mut self, _context: &mut Self::Context) -> Result<()> {
        info!("AppWorker::initialize");
        Ok(())
    }

    async fn shutdown(&mut self, _context: &mut Self::Context) -> Result<()> {
        info!("AppWorker::shutdown");
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Self::Context,
        msg: Routed<Self::Message>,
    ) -> Result<()> {
        info!(
            "AppWorker::handle_message: {} Received: {}",
            ctx.address(),
            msg
        );

        // Echo the message body back on its return_route.
        ctx.send(msg.return_route(), "{ 'some': 'result' }".to_string())
            .await
    }
}
