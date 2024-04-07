#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

pub use self::behaviour::{Behaviour, Config, Event};
pub use self::protocol::{Info, UpgradeError, PROTOCOL_NAME, PUSH_PROTOCOL_NAME};

mod behaviour;
mod handler;
mod protocol;

mod proto {
    #![allow(unreachable_pub)]
    include!("generated/mod.rs");
    pub(crate) use self::structs::Identify;
}
