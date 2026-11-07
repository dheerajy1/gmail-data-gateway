use dioxus::prelude::*;

use crate::components::icons::{Lock, User};
use crate::components::layout::Layout;

#[derive(Props, Clone, PartialEq)]
pub struct LoginProps {
    pub title: String,
}

// Login page rendered inside the shared Layout
#[component]
pub fn Login(props: LoginProps) -> Element {
    rsx! {
        Layout { title: props.title.clone(),
            div { class: "flex items-center justify-center min-h-screen bg-bg px-4",
                div { class: "w-full max-w-(--width-login-sm) md:max-w-(--width-login) bg-surface border border-border-thick p-8 md:p-12",
                    header { class: "text-center mb-6 md:mb-8",
                        div { class: "inline-flex items-center justify-center px-3 py-2 border border-logo-border bg-logo-bg text-logo-text font-bold text-2xl md:text-3xl mb-4",
                            "Gmail data gateway API"
                        }
                        h2 { class: "text-xl md:text-2xl font-medium uppercase tracking-(--tracking-wider) text-text-primary",
                            "Gateway Login"
                        }
                    }

                    form {
                        method: "POST",
                        class: "flex flex-col gap-4 md:gap-5 mt-6 md:mt-8",
                        div { class: "relative",
                            User { class: "absolute left-3 top-1/2 -translate-y-1/2 text-text-muted w-4 h-4 md:w-5 md:h-5" }
                            input {
                                name: "username",
                                placeholder: "Username",
                                required: true,
                                class: "w-full bg-bg border border-border text-text-primary pl-10 pr-3 py-3 md:py-3.5 text-sm md:text-base focus:outline-none focus:border-focus-border placeholder:text-text-muted",
                            }
                        }

                        div { class: "relative",
                            Lock { class: "absolute left-3 top-1/2 -translate-y-1/2 text-text-muted w-4 h-4 md:w-5 md:h-5" }
                            input {
                                name: "password",
                                r#type: "password",
                                placeholder: "Password",
                                required: true,
                                class: "w-full bg-bg border border-border text-text-primary pl-10 pr-3 py-3 md:py-3.5 text-sm md:text-base focus:outline-none focus:border-focus-border placeholder:text-text-muted",
                            }
                        }

                        button {
                            r#type: "submit",
                            class: "w-full mt-2 py-3 md:py-3.5 bg-button-bg text-button-text border border-border-thick font-semibold tracking-(--tracking-wider) transition-all duration-150 hover:bg-button-hover active:bg-button-active active:scale-95",
                            "SIGN IN"
                        }
                    }

                    div { class: "mt-6 text-center text-text-muted text-xs md:text-sm",
                        "Secure access to portal / API documentation"
                    }

                    footer { class: "mt-6 md:mt-8 pt-4 border-t border-double border-border-thick text-center text-text-muted text-xs",
                        p { "Developer: Dheeraj" }
                        p {
                            a {
                                href: "https://dheerajy1.hashnode.dev/",
                                class: "text-text-primary hover:underline",
                                "dheerajy1.hashnode.dev"
                            }
                        }
                    }
                }
            }
        }
    }
}
