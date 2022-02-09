use core::fmt;
use ockam_core::compat::collections::HashMap;
use ockam_core::{async_trait, Result};

// What is a Unique Unforgeable Reference? (UUR)
//
// A UUR can be:
//
//   - a memory location
//   - a reference that can be resolved to a memory location
//   - a reference that can be resolved to a network location that can resolve to a memory location
//   - a public key that can be verified and then be resolved ... to a memory location
//   - a reference to a memory location that can verify and then resolve ... to a memory location
//
// So a UUR represents something that, one way or another, will be
// resolved to a memory location that receives and responds to the
// sender's message.
//

/// UniqueUnforgeableReference
#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct MyUniqueUnforgeableReference(pub u128);

impl fmt::Debug for MyUniqueUnforgeableReference {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&format!("uur:{:032x})", self.0))
    }
}

impl PartialEq for MyUniqueUnforgeableReference {
    fn eq(&self, rhs: &MyUniqueUnforgeableReference) -> bool {
        self.0 == rhs.0
    }
}

/// MyCapability
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct MyCapability {
    /// The unique unforgeable reference that represents this capability
    pub uur: MyUniqueUnforgeableReference,
    /// A human-friendly name for this capability
    pub name: String,
    // TODO expires: DateTime
}

impl fmt::Debug for MyCapability {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&format!("{}@{:?}", self.name, self.uur))
    }
}

impl fmt::Display for MyCapability {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&format!("{}@{:?}", self.name, self.uur))
    }
}

/// MyCapabilities
pub type MyCapabilities = HashMap<&'static str, MyCapability>;

// - MyCapabilityRequest ---------------------------------------------------------------

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum MyCapabilityRequest {
    // requests
    IntroduceMe(MyCapability, String), // (MyCapability, "cap_name")
    OhHaiBob(MyCapability),
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum MyCapabilityResponse {
    Unauthorized,
    Introduction(MyCapability),
    OhHaiCarol,
}

impl ockam_core::Message for MyCapabilityRequest {}
impl ockam_core::Message for MyCapabilityResponse {}

// - new stuff ----------------------------------------------------------------

// is_authorized(subject, verb, object) -> bool
// is_authorized(subject, action, resource) -> bool

pub enum AuthResult {
    Deny = 0,
    Allow = 1,
}

// request capability
#[async_trait]
pub trait Authenticate {
    async fn request() -> Result<bool> {
        ockam_core::deny()
    }
}

// use capability
#[async_trait]
pub trait Authorize {
    async fn is_authorized(
        subject: MyUniqueUnforgeableReference, // my uur
        verb: MyUniqueUnforgeableReference,    // the capability I want
        object: MyUniqueUnforgeableReference,  // the thing that has the capability
    ) -> Result<bool> {
        ockam_core::deny()
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct _MyCapabilityRequest {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct _MyCapabilityResponse {}
