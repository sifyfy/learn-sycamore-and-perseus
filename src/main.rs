use sycamore::{builder::prelude::*, prelude::*, rt::JsCast};

fn main() {
    sycamore::render(|cx| {
        let section_1 = view! {cx,
            section(style="color:#f00") { "This is a section 1" }
        };

        // let section_2 = view! {cx,
        //     section(
        //         style="color:#0f0",
        //         on:click=alert
        //     ) { "This is a section 2" }
        // };
        let section_2 = clickable_section(cx, "くりっくめー");

        let state = create_signal(cx, 0);

        // これはcxのライフタイム境界を越えて使いたいのでRcSignalにする。
        // つまりSuspenseでは対応できない非同期処理で値を受け渡す時に使う。
        //  set_timeoutやrequest_animation_frameなどなど。
        let state2 = create_rc_signal(0);

        create_effect(cx, || {
            web_sys::console::log_1(&format!("State is {}", state.get()).into())
        });

        {
            let state2 = state2.clone();
            create_effect(cx, move || {
                state.track();
                // stateが変更された時、まず最初に変更後の値をキャッシュする。
                let state = *state.get();
                let state2 = state2.clone();
                sycamore::futures::spawn_local(async move {
                    web_sys::console::log_1(&format!("Before modify state2 (A)").into());
                    let _ = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(
                        &mut |accept, _| {
                            let f = std::rc::Rc::new(std::cell::RefCell::new(None));
                            let g = f.clone();

                            *g.borrow_mut() =
                                Some(wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                                    let _ = accept.call0(&0.into()).unwrap();
                                    let _ = f.borrow_mut().take();
                                })
                                    as Box<dyn FnMut()>));

                            let _ = web_sys::window()
                                .unwrap()
                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                    g.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                                    5000,
                                );
                        },
                    ))
                    .await;
                    web_sys::console::log_1(&format!("Before modify state2 (B)").into());
                    *state2.modify() = state * 2;
                });
                web_sys::console::log_1(&format!("state = {}", state).into());
            });
        }

        let section_3 = view! {cx,
            section(
                on:click=|_| { state.set(*state.get() + 1); },
            ) { (state.get()) }
        };

        *state.modify() += 1;

        let section_4 = view! {cx,
            section { (state.get()) }
        };

        let x = create_memo(cx, || *state.get() * 2);

        web_sys::console::log_1(&"Before rendering".into());

        // let f = move || {
        //     if *state.get() % 10 == 0 {
        //         view! { cx, div { "ten!!" } }
        //     } else {
        //         view! { cx, MyComponent(number=*state.get()) { "piyo" } }
        //     }
        // };

        let v = view! { cx, MyComponent(number=*state.get()) { "piyo" (*state.get()) } };

        view! { cx,
            p { (hello_world().trim_end_matches("!")) }
            (section_1)
            (section_2)
            (section_3)
            (section_4)
            div { "State2 = " (state2.get()) }
            div { (3-1) }
            div { button(class="btn btn-outline") { "牡丹" } }
            div { (x.get()) }
            div { input(type="number", value=*state.get()) {} }
            // (f())
            (v)
            // ({view!{cx, MyComponent(number=*state.get()) { "INU" }}})
            // ({ let x = *state.get(); view! {cx, MyComponent(number=x) { "neko" }} })
        }
    });
}

fn hello_world() -> &'static str {
    "Hello world!！!!"
}

fn alert(_: web_sys::Event) {
    web_sys::window()
        .unwrap()
        .alert_with_message("Clicked!!")
        .unwrap();
}

fn clickable_section<G: GenericNode<EventType = web_sys::Event>>(
    cx: Scope<'_>,
    text: &str,
) -> View<G> {
    section()
        .attr("style", "color:#0f0")
        .on("click", alert)
        .c(t(text))
        .view(cx)
}

#[derive(Debug, Prop)]
struct MyComponentProps<'a, G: Html> {
    number: i32,
    children: Children<'a, G>,
}

#[component]
fn MyComponent<'a, G: Html>(cx: Scope<'a>, props: MyComponentProps<'a, G>) -> View<G> {
    let children = props.children.call(cx);
    let subject = create_signal(cx, String::new());
    web_sys::console::log_1(&"これは繰り返し出る？".into());
    create_effect(cx, move || {
        *subject.modify() = format!("Current number is {}", props.number);
    });
    on_cleanup(cx, || {
        web_sys::console::log_1(&"MyComponent is destroyed".into())
    });
    view! { cx,
        div {
            div { (subject.get()) }
            div { (children) }
        }
    }
}
