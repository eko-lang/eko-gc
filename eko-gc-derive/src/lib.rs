#[macro_use]
extern crate synstructure;
#[macro_use]
extern crate quote;

fn trace_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
    s.gen_impl(quote! {
        extern crate eko_gc;

        use eko_gc::Trace;

        gen unsafe impl Trace for @Self {}
    })
}

decl_derive!([Trace] => trace_derive);