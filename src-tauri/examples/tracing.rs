use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main(){
    tracing_subscriber::fmt()
    .event_format(
        tracing_subscriber::fmt::format()
            .with_file(true)
            .with_line_number(true)
    )
    .init();
    
    let foo = 42;
    tracing::info!(foo, "Hello from tracing");
}