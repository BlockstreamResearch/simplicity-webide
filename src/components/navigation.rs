use leptos::{component, create_signal, view, IntoView, SignalGet, SignalSet};

#[component]
pub fn Navigation() -> impl IntoView {
    let (mobile_menu_open, set_mobile_menu_open) = create_signal(false);
    let (docs_open, set_docs_open) = create_signal(false);
    let (packages_open, set_packages_open) = create_signal(false);
    let (learn_open, set_learn_open) = create_signal(false);
    let (community_open, set_community_open) = create_signal(false);

    let toggle_mobile_menu = move |_| {
        set_mobile_menu_open.set(!mobile_menu_open.get());
        if !mobile_menu_open.get() {
            set_docs_open.set(false);
            set_packages_open.set(false);
            set_learn_open.set(false);
            set_community_open.set(false);
        }
    };

    let toggle_docs = move |_| {
        set_docs_open.set(!docs_open.get());
    };

    let toggle_packages = move |_| {
        set_packages_open.set(!packages_open.get());
    };

    let toggle_learn = move |_| {
        set_learn_open.set(!learn_open.get());
    };

    let toggle_community = move |_| {
        set_community_open.set(!community_open.get());
    };

    let close_mobile_menu = move |_| {
        set_mobile_menu_open.set(false);
        set_docs_open.set(false);
        set_packages_open.set(false);
        set_learn_open.set(false);
        set_community_open.set(false);
    };

    view! {
        <nav class="navigation">
            <div class="navigation-container">
                <a href="/" class="navigation-logo-link">
                    <img src="images/simplicity-logo.svg" alt="Simplicity Logo" class="navigation-logo" />
                </a>

                // Desktop Navigation
                <div class="navigation-desktop">
                    <div class="navigation-dropdown">
                        <span class="navigation-dropdown-label">"Docs"</span>
                        <div class="navigation-dropdown-content">
                            <a href="https://blockstream.com/simplicity.pdf" target="_blank" rel="noopener noreferrer" class="navigation-dropdown-item">
                                <div class="navigation-dropdown-title">"Whitepaper"</div>
                                <div class="navigation-dropdown-description">"The original Simplicity whitepaper, detailing the language."</div>
                            </a>
                            <a href="https://docs.simplicity-lang.org/documentation/jets-explained/" target="_blank" rel="noopener noreferrer" class="navigation-dropdown-item">
                                <div class="navigation-dropdown-title">"Jets Documentation"</div>
                                <div class="navigation-dropdown-description">"Technical reference detailing each jet's functionality and use."</div>
                            </a>
                            <a href="https://docs.simplicity-lang.org/" target="_blank" rel="noopener noreferrer" class="navigation-dropdown-item">
                                <div class="navigation-dropdown-title">"SimplicityHL"</div>
                                <div class="navigation-dropdown-description">"A high-level, developer-friendly language that compiles down to Simplicity."</div>
                            </a>
                            <a href="https://github.com/BlockstreamResearch/SimplicityHL/tree/master/examples" target="_blank" rel="noopener noreferrer" class="navigation-dropdown-item">
                                <div class="navigation-dropdown-title">"Contract Examples"</div>
                                <div class="navigation-dropdown-description">"A curated set of smart contract templates and patterns."</div>
                            </a>
                        </div>
                    </div>

                    <div class="navigation-dropdown">
                        <span class="navigation-dropdown-label">"Packages"</span>
                        <div class="navigation-dropdown-content">
                            <a href="https://github.com/BlockstreamResearch/simplicity" target="_blank" rel="noopener noreferrer" class="navigation-dropdown-item">
                                <div class="navigation-dropdown-title">"Simplicity"</div>
                                <div class="navigation-dropdown-description">"The low-level, formally-verifiable smart contract language."</div>
                            </a>
                            <a href="https://github.com/BlockstreamResearch/SimplicityHL" target="_blank" rel="noopener noreferrer" class="navigation-dropdown-item">
                                <div class="navigation-dropdown-title">"SimplicityHL Compiler"</div>
                                <div class="navigation-dropdown-description">"The toolchain that translates SimplicityHL code into raw Simplicity."</div>
                            </a>
                            <a href="https://ide.simplicity-lang.org/" class="navigation-dropdown-item">
                                <div class="navigation-dropdown-title">"SimplicityHL IDE"</div>
                                <div class="navigation-dropdown-description">"A browser-based playground for writing and testing contracts."</div>
                            </a>
                        </div>
                    </div>

                    <div class="navigation-dropdown">
                        <span class="navigation-dropdown-label">"Learn"</span>
                        <div class="navigation-dropdown-content">
                            <a href="https://simplicity-lang.org/articles" class="navigation-dropdown-item">
                                <div class="navigation-dropdown-title">"Articles"</div>
                                // <div class="navigation-dropdown-description">"Read in-depth articles about Simplicity and smart contracts."</div>
                            </a>
                            <a href="https://simplicity-lang.org/videos" class="navigation-dropdown-item">
                                <div class="navigation-dropdown-title">"Videos"</div>
                                // <div class="navigation-dropdown-description">"Watch tutorials and presentations about Simplicity."</div>
                            </a>
                        </div>
                    </div>

                    <div class="navigation-dropdown navigation-dropdown-community">
                        <span class="navigation-dropdown-label">"Community"</span>
                        <div class="navigation-dropdown-content navigation-dropdown-content-community">
                            <a href="https://x.com/blksresearch" target="_blank" rel="noopener noreferrer" class="navigation-dropdown-item navigation-dropdown-item-community">
                                <img src="images/x.svg" alt="Twitter" class="navigation-dropdown-icon" />
                                <span class="navigation-dropdown-title">"Twitter"</span>
                            </a>
                            <a href="https://t.me/simplicity_community" target="_blank" rel="noopener noreferrer" class="navigation-dropdown-item navigation-dropdown-item-community">
                                <img src="images/telegram.svg" alt="Telegram" class="navigation-dropdown-icon" />
                                <span class="navigation-dropdown-title">"Telegram"</span>
                            </a>
                        </div>
                    </div>
                </div>

                // Mobile Menu Button
                <button class="navigation-mobile-button" on:click=toggle_mobile_menu>
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z"/>
                    </svg>
                </button>
            </div>

            // Mobile Menu
            <div
                class="navigation-mobile-menu"
                class:active=move || mobile_menu_open.get()
            >
                <div class="navigation-mobile-section">
                    <button class="navigation-mobile-dropdown-button" on:click=toggle_docs>"Docs" <i class="fa fa-chevron-down"></i></button>
                    <div
                        class="navigation-mobile-dropdown"
                        class:active=move || docs_open.get()
                    >
                        <a href="https://blockstream.com/simplicity.pdf" target="_blank" rel="noopener noreferrer" class="navigation-mobile-dropdown-item" on:click=close_mobile_menu>"Whitepaper"</a>
                        <a href="https://docs.simplicity-lang.org/documentation/jets-explained/" target="_blank" rel="noopener noreferrer" class="navigation-mobile-dropdown-item" on:click=close_mobile_menu>"Jets Documentation"</a>
                        <a href="https://docs.simplicity-lang.org/" target="_blank" rel="noopener noreferrer" class="navigation-mobile-dropdown-item" on:click=close_mobile_menu>"SimplicityHL"</a>
                        <a href="https://github.com/BlockstreamResearch/SimplicityHL/tree/master/examples" target="_blank" rel="noopener noreferrer" class="navigation-mobile-dropdown-item" on:click=close_mobile_menu>"Contract Examples"</a>
                    </div>
                </div>

                <div class="navigation-mobile-section">
                    <button class="navigation-mobile-dropdown-button" on:click=toggle_packages>"Packages" <i class="fa fa-chevron-down"></i></button>
                    <div
                        class="navigation-mobile-dropdown"
                        class:active=move || packages_open.get()
                    >
                        <a href="https://github.com/BlockstreamResearch/simplicity" target="_blank" rel="noopener noreferrer" class="navigation-mobile-dropdown-item" on:click=close_mobile_menu>"Simplicity"</a>
                        <a href="https://github.com/BlockstreamResearch/SimplicityHL" target="_blank" rel="noopener noreferrer" class="navigation-mobile-dropdown-item" on:click=close_mobile_menu>"SimplicityHL Compiler"</a>
                        <a href="https://ide.simplicity-lang.org/" class="navigation-mobile-dropdown-item" on:click=close_mobile_menu>"SimplicityHL IDE"</a>
                    </div>
                </div>

                <div class="navigation-mobile-section">
                    <button class="navigation-mobile-dropdown-button" on:click=toggle_learn>"Learn" <i class="fa fa-chevron-down"></i></button>
                    <div
                        class="navigation-mobile-dropdown"
                        class:active=move || learn_open.get()
                    >
                        <a href="https://simplicity-lang.org/articles" class="navigation-mobile-dropdown-item" on:click=close_mobile_menu>"Articles"</a>
                        <a href="https://simplicity-lang.org/videos" class="navigation-mobile-dropdown-item" on:click=close_mobile_menu>"Videos"</a>
                    </div>
                </div>

                <div class="navigation-mobile-section">
                    <button class="navigation-mobile-dropdown-button" on:click=toggle_community>"Community" <i class="fa fa-chevron-down"></i></button>
                    <div
                        class="navigation-mobile-dropdown"
                        class:active=move || community_open.get()
                    >
                        <a href="https://x.com/blksresearch" target="_blank" rel="noopener noreferrer" class="navigation-mobile-dropdown-item navigation-mobile-dropdown-item-community" on:click=close_mobile_menu>
                            <img src="images/x.svg" alt="Twitter" class="navigation-dropdown-icon" />
                            <span>"Twitter"</span>
                        </a>
                        <a href="https://t.me/simplicity_community" target="_blank" rel="noopener noreferrer" class="navigation-mobile-dropdown-item navigation-mobile-dropdown-item-community" on:click=close_mobile_menu>
                            <img src="images/telegram.svg" alt="Telegram" class="navigation-dropdown-icon" />
                            <span>"Telegram"</span>
                        </a>
                    </div>
                </div>
            </div>
        </nav>
    }
}
