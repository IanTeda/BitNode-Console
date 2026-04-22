use leptos::prelude::*;

fn main() {
    // Set a panic hook to get better error messages in the browser console
    console_error_panic_hook::set_once();
    
    leptos::mount::mount_to_body(|| view! { <p>"Hello, world!"</p> })
}