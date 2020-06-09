use prometheus::{register_int_counter, register_int_gauge, IntCounter, IntGauge};

// To call in main().
pub fn init() {
    // Access once to trigger the registering.
    let _ = (
        REQUESTS.get(),
        RESPONSES.get(),
        CONNECTIONS.get(),
        CONCURRENT_USERS.get(),
    );
}

lazy_static::lazy_static! {
    // To REQUESTS.inc() each time a request to game server is received.
    pub static ref REQUESTS: IntCounter =
        register_int_counter!(
            "metrics_gateway_requests_total",
            "The total number of requests to game server from users"
        )
        .unwrap();

    // To RESPONSES.inc() each time a response from game server is sent, including push messages.
    pub static ref RESPONSES: IntCounter =
        register_int_counter!(
            "metrics_gateway_responses_total",
            "The total number of responses from game server to users, including push messages"
        )
        .unwrap();

    // To CONNECTIONS.inc() each time a new connection is established.
    pub static ref CONNECTIONS: IntCounter =
        register_int_counter!(
            "metrics_gateway_connections_total",
            "The total number of connections from users"
        )
        .unwrap();

    // To CONCURRENT_USERS.inc() in c-tor, to CONCURRENT_USERS.dec() in drop().
    pub static ref CONCURRENT_USERS: IntGauge =
        register_int_gauge!(
            "metrics_gateway_concurrent_users",
            "The number of users simultaneously logged in to a session"
        )
        .unwrap();
}
