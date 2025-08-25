use leptos::{component, create_signal, view, wasm_bindgen, IntoView, SignalGet, SignalSet};
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="footer-container">
                <div class="footer-content">
                    <a href="https://simplicity-lang.org" class="footer-logo-link">
                        <img src="images/simplicity-logo.svg" alt="Simplicity Logo" class="footer-logo" />
                    </a>
                    <h3 class="footer-title">
                        "Stay in the Loop"
                    </h3>
                    <p class="footer-description">
                        "Get the latest updates on Simplicity's progress and features."
                    </p>
                    <NewsletterForm />
                </div>

                <div class="footer-bottom">
                    <div class="footer-links">
                        <a href="https://blockstream.com/terms/" class="footer-link">
                            "Terms"
                        </a>
                        <a href="https://blockstream.com/privacy" class="footer-link">
                            "Privacy"
                        </a>
                    </div>

                    <div class="footer-social">
                        <a href="https://x.com/blksresearch" class="footer-social-link">
                            <img src="images/x.svg" alt="X Icon" class="footer-social-icon" />
                        </a>
                        <a href="https://t.me/simplicity_community" class="footer-social-link">
                            <img src="images/telegram.svg" alt="Telegram Icon" class="footer-social-icon" />
                        </a>
                        <a href="https://github.com/BlockstreamResearch/simplicityhl" class="footer-social-link">
                            <img src="images/github.svg" alt="GitHub Icon" class="footer-social-icon" />
                        </a>
                    </div>
                </div>
            </div>
        </footer>
    }
}

#[component]
fn NewsletterForm() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (status, set_status) = create_signal(FormStatus::Idle);
    let (message, set_message) = create_signal(String::new());

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let email_value = email.get();
        if email_value.is_empty() || !email_value.contains('@') {
            set_status.set(FormStatus::Error);
            set_message.set("Please enter a valid email address".to_string());
            return;
        }

        set_status.set(FormStatus::Loading);

        spawn_local(async move {
            match subscribe_to_newsletter(email_value).await {
                Ok(()) => {
                    set_status.set(FormStatus::Success);
                    set_message
                        .set("Thank you for subscribing! We'll keep you updated.".to_string());
                    set_email.set(String::new());
                }
                Err(err) => {
                    set_status.set(FormStatus::Error);
                    set_message.set(err);
                }
            }
        });
    };

    view! {
        <form class="newsletter-form" on:submit=handle_submit>
            <div class="newsletter-input-container">
                <img src="images/envelope.svg" alt="Envelope Icon" class="newsletter-input-icon" />
                <input
                    type="email"
                    name="email"
                    prop:value=move || email.get()
                    on:input=move |ev| set_email.set(event_target_value(&ev))
                    placeholder="Email address"
                    class="newsletter-input"
                    required
                />
            </div>
            <button
                type="submit"
                class="newsletter-button"
                disabled=move || matches!(status.get(), FormStatus::Loading)
            >
                {move || match status.get() {
                    FormStatus::Loading => "Subscribing...",
                    _ => "Subscribe →"
                }}
            </button>

            {move || match status.get() {
                FormStatus::Success => view! {
                    <p class="newsletter-message newsletter-message-success">{message.get()}</p>
                }.into_view(),
                FormStatus::Error => view! {
                    <p class="newsletter-message newsletter-message-error">{message.get()}</p>
                }.into_view(),
                _ => view! {}.into_view()
            }}
        </form>
    }
}

#[derive(Clone, PartialEq)]
enum FormStatus {
    Idle,
    Loading,
    Success,
    Error,
}

async fn subscribe_to_newsletter(email: String) -> Result<(), String> {
    use gloo_net::http::Request;
    use serde_json::json;

    let api_url = "https://simplicity-lang.org/api/subscribe";

    let response = Request::post(api_url)
        .header("Content-Type", "application/json")
        .json(&json!({ "email": email }))
        .map_err(|e| format!("Failed to create request: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if response.ok() {
        Ok(())
    } else {
        match response.json::<serde_json::Value>().await {
            Ok(error_data) => {
                let error_msg = error_data
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Subscription failed");
                Err(error_msg.to_string())
            }
            Err(_) => Err("Subscription failed. Please try again.".to_string()),
        }
    }
}

fn event_target_value(ev: &leptos::ev::Event) -> String {
    ev.target()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>()
        .value()
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_local<F>(_future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
}
