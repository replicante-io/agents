use replicante_util_actixweb::APIFlags;
use replicante_util_actixweb::RootDescriptor;

/// Enumerates all possible API roots.
///
/// All endpoints must fall under one of these roots and are subject to all restrictions
/// of that specific root.
/// The main restriction is that versioned APIs are subject to semver guarantees.
pub enum APIRoot {
    /// API root for all endpoints that are not yet stable.
    ///
    /// Endpoints in this root are NOT subject to ANY compatibility guarantees!
    UnstableAPI,

    /// Instrospection APIs not yet stable.
    UnstableIntrospect,
}

impl RootDescriptor for APIRoot {
    fn enabled(&self, flags: &APIFlags) -> bool {
        match self {
            APIRoot::UnstableAPI => match flags.get("unstable") {
                Some(flag) => *flag,
                None => true,
            },
            APIRoot::UnstableIntrospect => match flags.get("unstable") {
                Some(flag) if !flag => false,
                _ => match flags.get("introspect") {
                    Some(flag) => *flag,
                    None => true,
                },
            },
        }
    }

    fn prefix(&self) -> &'static str {
        match self {
            APIRoot::UnstableAPI => "/api/unstable",
            APIRoot::UnstableIntrospect => "/api/unstable/introspect",
        }
    }
}
