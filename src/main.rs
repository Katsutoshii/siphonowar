use sipho::prelude::*;

#[cfg(feature = "debug")]
mod debug;

fn main() {
    let mut app = App::new();
    app.add_plugins(SiphonowarPlugin);
    #[cfg(feature = "debug")]
    {
        app.add_plugins(debug::DebugPlugin);
    }
    app.run();
}
