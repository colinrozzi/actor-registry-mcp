mod bindings;

use crate::bindings::exports::ntwk::theater::actor::Guest;
use crate::bindings::exports::ntwk::theater::message_server_client::Guest as MessageServerClient;
use crate::bindings::ntwk::theater::runtime::log;
use crate::bindings::ntwk::theater::types::State;

struct {{actor_name}};
impl Guest for {{actor_name}} {
    fn init(_state: State, params: (String,)) -> Result<(State,), String> {
        log("Initializing {{actor_name}} actor");
        let (param,) = params;
        log(&format!("Init parameter: {}", param));

        log("Hello from {{actor_name}}!");

        Ok((Some(vec![]),))
    }
}

impl MessageServerClient for {{actor_name}} {
    fn handle_send(
        state: Option<Vec<u8>>,
        params: (Vec<u8>,),
    ) -> Result<(Option<Vec<u8>>,), String> {
        log("Handling send message");
        let (data,) = params;
        log(&format!("Received data: {:?}", data));
        Ok((state,))
    }

    fn handle_request(
        state: Option<Vec<u8>>,
        params: (String, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>, (Option<Vec<u8>>,)), String> {
        log("Handling request message");
        let (request_id, data) = params;
        log(&format!(
            "[req id] {} [data] {}",
            request_id,
            String::from_utf8(data.clone()).expect("Failed to convert data to string")
        ));

        Ok((state, (Some(data),)))
    }

    fn handle_channel_open(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (bindings::exports::ntwk::theater::message_server_client::Json,),
    ) -> Result<
        (
            Option<bindings::exports::ntwk::theater::message_server_client::Json>,
            (bindings::exports::ntwk::theater::message_server_client::ChannelAccept,),
        ),
        String,
    > {
        log("Handling channel open message");
        log(&format!("Channel open message: {:?}", params));
        Ok((
            state,
            (
                bindings::exports::ntwk::theater::message_server_client::ChannelAccept {
                    accepted: true,
                    message: None,
                },
            ),
        ))
    }

    fn handle_channel_close(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (String,),
    ) -> Result<(Option<bindings::exports::ntwk::theater::message_server_client::Json>,), String>
    {
        log("Handling channel close message");
        log(&format!("Channel close message: {:?}", params));
        Ok((state,))
    }

    fn handle_channel_message(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (
            String,
            bindings::exports::ntwk::theater::message_server_client::Json,
        ),
    ) -> Result<(Option<bindings::exports::ntwk::theater::message_server_client::Json>,), String>
    {
        log("Received channel message");
        log(&format!("Channel message: {:?}", params));
        Ok((state,))
    }
}

bindings::export!({{actor_name}} with_types_in bindings);
