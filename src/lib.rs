extern crate proc_macro;

use darling::{FromMeta, ToTokens};
use proc_macro::TokenStream;
use syn::{self, parse_macro_input, parse_quote, AttributeArgs, ItemFn, Stmt};

#[derive(FromMeta, Debug)]
struct MacroArgs {
    port: i32,
    instances: usize,
}

#[proc_macro_attribute]
pub fn server(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let mut input = parse_macro_input!(input as ItemFn);

    let attr_args = match MacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    impl_server(&attr_args, &mut input)
}

fn impl_server(attr_args: &MacroArgs, input: &mut ItemFn) -> TokenStream {
    let fn_name = &input.sig.ident;

    let port = attr_args.port;
    let instances = attr_args.instances;

    if fn_name != "main" {
        panic!("server macro is only applicable for main function!");
    }

    let mut statements: Vec<Stmt> = vec![];

    statements.push(parse_quote! {
        use std::net::TcpListener;
    });

    statements.push(parse_quote! {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", #port.to_string())).unwrap();
    });

    statements.push(parse_quote! {
        let pool = ThreadPool::new(#instances);
    });

    statements.push(parse_quote! {
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            pool.execute(|| handle_connection(stream));
        }
    });

    input.block.stmts.splice(0..0, statements);

    input.to_token_stream().into()
}
