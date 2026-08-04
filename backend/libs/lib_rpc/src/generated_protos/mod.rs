pub mod authentication {
    include!(concat!(
        env!("OUT_DIR"),
        "/bitnode_console.authentication.v1.rs"
    ));
}

pub mod bitcoin_daemon {
    include!(concat!(
        env!("OUT_DIR"),
        "/bitnode_console.bitcoin_daemon.v1.rs"
    ));
}

pub mod common {
    include!(concat!(env!("OUT_DIR"), "/bitnode_console.common.v1.rs"));
}

pub mod journals {
    include!(concat!(env!("OUT_DIR"), "/bitnode_console.journals.v1.rs"));
}

pub mod utilities {
    include!(concat!(env!("OUT_DIR"), "/bitnode_console.utilities.v1.rs"));
}
