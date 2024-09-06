#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use freya::prelude::*;
use std::collections::HashMap;

fn main() {
    launch_with_props(app, "Split Routing Example", (550.0, 400.0));
}

struct Document {
    name: String,
    content: String,
}

static DOCUMENTS: GlobalSignal<HashMap<String, Document>> =
    GlobalSignal::new(|| HashMap::default());

pub fn load_documents() {
    let map = HashMap::from([
        (
            "document_1".to_string(),
            Document {
                name: "Document 1".to_string(),
                content: "Content 1".to_string(),
            },
        ),
        (
            "document_2".to_string(),
            Document {
                name: "Document 2".to_string(),
                content: "Content 2".to_string(),
            },
        ),
    ]);

    *DOCUMENTS.write_unchecked() = map;
}

fn app() -> Element {
    use_hook(|| {
        // Initialize the documents when the app is started
        load_documents();
    });

    rsx!(tabbed_ui::TabContainer {})
}

mod tabbed_ui {
    use dioxus_router::prelude::{Outlet, Routable, Router};
    use freya::components::*;
    use freya::prelude::*;

    // Tabbed UI has dependencies on the tabs it uses
    use crate::document::DocumentContainer;
    use crate::DOCUMENTS;

    #[derive(Routable, Clone, PartialEq)]
    #[rustfmt::skip]
    pub enum TabsRoute {
        #[layout(TabLayout)]
        #[route("/")]
        HomeTab,
        #[route("/document/:id")]
        DocumentTab { id: String },
        #[end_layout]
        #[route("/..route")]
        PageNotFound { },
    }

    /// Note: this is the ONLY public function
    #[allow(non_snake_case)]
    #[component]
    pub fn TabContainer() -> Element {
        rsx!(Router::<TabsRoute> {})
    }

    #[allow(non_snake_case)]
    #[component]
    fn TabLayout() -> Element {
        let mut sorted_document_ids = DOCUMENTS
            .read()
            .iter()
            .map(|(key, doc)| (key.clone(), doc.name.clone()))
            .collect::<Vec<(String, String)>>()
            .clone();
        sorted_document_ids.sort();

        rsx!(
            NativeRouter {
                rect {
                    background: "#444444",
                    width: "fill",
                    Tabsbar {
                        Link {
                            to: TabsRoute::HomeTab,
                            ActivableRoute {
                                route: TabsRoute::HomeTab,
                                exact: true,
                                Tab {
                                    label {
                                        "Home"
                                    }
                                }
                            }
                        },
                        for (id, name) in sorted_document_ids {
                            Link {
                                key: "{id.clone()}",
                                to: TabsRoute::DocumentTab { id: id.clone() },
                                ActivableRoute {
                                    route: TabsRoute::DocumentTab { id },
                                    Tab {
                                        label {
                                            "{name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Body {
                    rect {
                        main_align: "center",
                        cross_align: "center",
                        width: "fill",
                        height: "fill",
                        Outlet::<TabsRoute> {  }
                    }
                }
            }
        )
    }

    #[allow(non_snake_case)]
    #[component]
    fn DocumentTab(id: String) -> Element {
        println!("DocumentTab. id: {}", id);

        rsx!(DocumentContainer { id })
    }

    #[allow(non_snake_case)]
    #[component]
    fn HomeTab() -> Element {
        rsx!(
            label {
                "Home Tab Content"
            }
        )
    }

    #[allow(non_snake_case)]
    #[component]
    fn PageNotFound() -> Element {
        rsx!(
            label {
                "404!! 😵"
            }
        )
    }
}

///
/// Document UI - should have know knowledge or dependencies on the tabbed UI
///
mod document {
    use std::ops::Deref;

    use crate::DOCUMENTS;
    use dioxus_router::{
        hooks::use_route,
        prelude::{Outlet, Routable, Router, RouterConfig},
    };
    use freya::prelude::*;

    static DOCUMENTS_ROUTER: GlobalSignal<DocumentRoute> =
        GlobalSignal::new(|| DocumentRoute::DocumentOverview);

    #[derive(Routable, Clone, PartialEq)]
    #[rustfmt::skip]
    enum DocumentRoute {
        #[layout(DocumentLayout)]
            #[route("/")]
            DocumentOverview,
            #[route("/content")]
            DocumentContent,
        #[end_layout]
        #[route("/..route")]
        DocumentPageNotFound {},
    }

    #[derive(Clone, PartialEq, Debug)]
    struct ActiveDocumentId(pub String);

    impl Deref for ActiveDocumentId {
        type Target = String;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    /// Note: this is the ONLY public function
    #[allow(non_snake_case)]
    #[component]
    pub fn DocumentContainer(id: String) -> Element {
        println!("DocumentContainer. id: {}", id);

        let active_id = use_memo(use_reactive(&id, |id| ActiveDocumentId(id)));

        use_context_provider(|| {
            let signal: ReadOnlySignal<ActiveDocumentId> = active_id.into();
            signal
        });

        rsx!(Router::<DocumentRoute> {
            config: || RouterConfig::default().initial_route(DOCUMENTS_ROUTER())
        })
    }

    #[allow(non_snake_case)]
    #[component]
    fn DocumentLayout() -> Element {
        let route = use_route::<DocumentRoute>();
        let id = use_context::<ReadOnlySignal<ActiveDocumentId>>();

        println!("DocumentLayout. id: {:?}", id);

        use_effect(use_reactive!(|route| {
            *DOCUMENTS_ROUTER.write_unchecked() = route;
            println!("UPDATED");
        }));

        rsx!(
            NativeRouter {
                Sidebar {
                    sidebar: rsx!(
                        Link {
                            to: DocumentRoute::DocumentOverview,
                            ActivableRoute {
                                route: DocumentRoute::DocumentOverview,
                                exact: true,
                                SidebarItem {
                                    label {
                                        "Overview"
                                    }
                                }
                            }
                        },
                        Link {
                            to: DocumentRoute::DocumentContent,
                            ActivableRoute {
                                route: DocumentRoute::DocumentContent,
                                SidebarItem {
                                    label {
                                        "Content"
                                    }
                                }
                            }
                        },
                    ),
                    Body {
                        rect {
                            main_align: "center",
                            cross_align: "center",
                            width: "fill",
                            height: "fill",
                            Outlet::<DocumentRoute> {  }
                        }
                    }
                }
            }
        )
    }

    #[allow(non_snake_case)]
    #[component]
    fn DocumentOverview() -> Element {
        let id = use_context::<ReadOnlySignal<ActiveDocumentId>>();
        println!("DocumentOverview. id: {:?} !!!", id);

        rsx!(
            label {
                "Overview. (path: '/', id: {id:?}) !"
            }
        )
    }

    #[allow(non_snake_case)]
    #[component]
    fn DocumentContent() -> Element {
        let id = use_context::<ReadOnlySignal<ActiveDocumentId>>();

        println!("DocumentContent. id: {:?}", id);

        let documents = DOCUMENTS.read();
        let document = documents.get(&id.read().0);

        match document {
            Some(document) => {
                rsx!(
                    label {
                        { format!("{}", document.content)}
                    }
                )
            }
            _ => {
                rsx!(
                    label {
                        "Error"
                    }
                )
            }
        }
    }

    #[allow(non_snake_case)]
    #[component]
    fn DocumentPageNotFound() -> Element {
        rsx!(
            label {
                "Document 404!! 😵"
            }
        )
    }
}
