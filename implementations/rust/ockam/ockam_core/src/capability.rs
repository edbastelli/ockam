use core::fmt;

use crate::compat::{boxed::Box, collections::BTreeMap, rand::random, string::String};
use crate::{AccessControl, AddressSet, LocalMessage, Result};

/// TODO
pub struct MessageFlowAuthorization {
    /// TODO
    pub address_set: AddressSet,
}

impl MessageFlowAuthorization {
    /// Create a new MessageFlowAuthorization access control
    pub fn new<S>(address_set: S) -> Self
    where
        S: Into<AddressSet>,
    {
        Self {
            address_set: address_set.into(),
        }
    }
}

#[crate::async_trait]
impl AccessControl for MessageFlowAuthorization {
    async fn is_authorized(&self, local_msg: &LocalMessage) -> Result<bool> {
        warn!(
            "MessageFlow::{}::is_authorized -> {}",
            self.address_set,
            local_msg.transport()
        );
        let payload = &local_msg.transport().payload[1..];
        warn!("\tPayload: {:?}", String::from_utf8_lossy(payload));
        crate::allow()
    }
}

#[crate::async_trait]
/// Capable
trait Capable {
    /// TODO
    async fn has_capability(&self, _subject: u64, _capability: &Action) -> Result<bool> {
        crate::deny()
    }
}

/// UniqueUnforgeableReference
#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct UniqueUnforgeableReference(pub u128);

impl Default for UniqueUnforgeableReference {
    /// Create a default UniqueUnforgeableReference
    fn default() -> Self {
        Self::new()
    }
}

impl UniqueUnforgeableReference {
    /// Create a new UniqueUnforgeableReference
    pub fn new() -> Self {
        Self(random())
    }
}

impl fmt::Debug for UniqueUnforgeableReference {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&format!("uur:{:032x})", self.0))
    }
}

impl PartialEq for UniqueUnforgeableReference {
    fn eq(&self, rhs: &UniqueUnforgeableReference) -> bool {
        self.0 == rhs.0
    }
}

/// Capability
#[derive(Clone)]
pub enum Action {
    /// Can send a message to the given UUR
    SendMessage,
}

/// CapabilityReference
#[derive(Clone)]
pub struct Capability {
    /// The capability
    pub action: Action,
    /// The address this capability is valid for
    pub address: AddressSet,
    /// The unique unforgeable reference that represents this capability
    pub uur: UniqueUnforgeableReference,
}

/// Capabilities
pub type Capabilities = BTreeMap<AddressSet, Capability>;

/// CapabilityAuthorization
pub struct CapabilityAuthorization {
    issued: Capabilities,
    received: Capabilities,
}

impl Default for CapabilityAuthorization {
    /// Create a default CapabilityAuthorization
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityAuthorization {
    /// Create a new CapabilityAuthorization
    pub fn new() -> Self {
        Self {
            issued: Capabilities::new(),
            received: Capabilities::new(),
        }
    }

    /// Request a capability for the given address
    pub async fn request<A>(&mut self, action: Action, address: A) -> Result<Capability>
    where
        A: Into<AddressSet>,
    {
        let address: AddressSet = address.into();

        // TODO check if we are willing?

        let capability_reference = Capability {
            action,
            address: address.clone(),
            uur: UniqueUnforgeableReference::new(),
        };
        self.issued.insert(address, capability_reference.clone());
        Ok(capability_reference)
    }

    /// Provision self with the given capability
    pub fn provision<A>(&mut self, capability: Capability, address: A) -> Result<()>
    where
        A: Into<AddressSet>,
    {
        self.received.insert(address.into(), capability);
        Ok(())
    }
}
