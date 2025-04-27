pub const WORLD_WIT: &str = r#"package ntwk:theater;

world {{actor_name}} {
    import runtime;
    import http-framework;
    import http-client;
    import websocket-types;

    export message-server-client;
    export actor;
    export http-handlers;
}
"#;
