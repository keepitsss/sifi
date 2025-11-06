use std::{collections::HashSet, sync::Arc};

use axum::{Router, routing::get};
use hyper::{Request, body::Incoming};
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        ServerConfig,
        pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    },
};
use tower_service::Service;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(main_page));

    // run our app with hyper, listening globally on port 3000
    let tcp_listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    // TODO: how it works?
    //
    // Especially how work tls and hyper?

    let tls_acceptor = TlsAcceptor::from(Arc::new(
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                Vec::from([CertificateDer::from_pem_file("secrets/cert.pem").unwrap()]),
                PrivateKeyDer::from_pem_file("secrets/key.pem").unwrap(),
            )
            .unwrap(),
    ));

    loop {
        let tower_service = app.clone();
        let tls_acceptor = tls_acceptor.clone();
        let (cnx, _addr) = tcp_listener.accept().await.unwrap();
        tokio::spawn(async move {
            let stream = tls_acceptor.accept(cnx).await.unwrap();
            let stream = TokioIo::new(stream);
            let hyper_service = hyper::service::service_fn(move |req: Request<Incoming>| {
                tower_service.clone().call(req)
            });
            let ret = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(stream, hyper_service)
                .await;
            if let Err(_err) = ret {
                todo!()
            }
        });
    }
}

static GLOBAL_STYLES: &str = include_str!("style.css");
use bumpalo::Bump;
use lib_html::*;

async fn main_page() -> axum::response::Html<String> {
    let arena = Bump::new();
    let arena: &Bump = &arena;

    let body = body(arena).child("Hello World!");

    render_page(arena, body).into()
}

fn render_page<'a>(allocator: &'a Bump, body: Body<'a>) -> String {
    let mut html = html(allocator);
    html.body(body);

    let mut cx = Context {
        indentation: utils::Indentation::default(),
        output: String::new(),
        arena: allocator,
        ids: HashSet::new(),
        styles: HashSet::new(),
    };

    cx.styles.extend([GLOBAL_STYLES]);

    html.render(&mut cx);

    let output = cx.output;
    drop(html);
    output
}
