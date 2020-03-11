use std::sync::mpsc::sync_channel;
use std::sync::Arc;

use actix_web::middleware;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use failure::ResultExt;
use futures::future::Future;
use humthreads::Builder;
use openssl::ssl::SslAcceptor;
use openssl::ssl::SslFiletype;
use openssl::ssl::SslMethod;
use openssl::ssl::SslVerifyMode;
use slog::info;

use replicante_util_actixweb::LoggingMiddleware;
use replicante_util_actixweb::MetricsMiddleware;
use replicante_util_actixweb::RootDescriptor;
use replicante_util_actixweb::SentryMiddleware;
use replicante_util_upkeep::Upkeep;

mod actions;
mod agent;
mod index;
mod introspect;
mod roots;

use crate::config::SentryCaptureApi;
use crate::metrics::REQUESTS;
use crate::Agent;
use crate::AgentContext;
use crate::ErrorKind;
use crate::Result;

pub use self::roots::APIRoot;

/// Additional API server configuration handlers.
#[derive(Clone)]
pub struct ApiAddons<T> {
    addons: Vec<Arc<dyn Fn(&mut web::ServiceConfig, &T) + Send + Sync>>,
}

impl<T> ApiAddons<T> {
    /// Run all the register handles to configure the given app.
    pub fn configure_app(&self, app: &mut web::ServiceConfig, context: &T) {
        for addon in &self.addons {
            addon(app, context);
        }
    }

    /// Register an app configuration function to be run later.
    pub fn register<F>(&mut self, addon: F)
    where
        F: Fn(&mut web::ServiceConfig, &T) + 'static + Send + Sync,
    {
        self.addons.push(Arc::new(addon));
    }
}

impl<T> Default for ApiAddons<T> {
    fn default() -> Self {
        ApiAddons { addons: Vec::new() }
    }
}

/// Mount all API endpoints.
fn configure_app(agent: Arc<dyn Agent>, context: AgentContext) -> impl Fn(&mut web::ServiceConfig) {
    move |app| {
        // Create the index root for each API root.
        let flags = context.config.api.trees.clone().into();
        let roots = [APIRoot::UnstableAPI];
        for root in roots.iter() {
            root.and_then(&flags, |root| {
                app.service(root.resource("/").route(web::get().to(index::index)));
            });
        }

        // Mount other roots.
        actions::configure_app(&flags, app, &context);
        agent::configure_app(&flags, app, Arc::clone(&agent), &context);
        introspect::configure_app(&context, &flags, app);
        context.api_addons.configure_app(app, &context);
    }
}

/// Start an Iron HTTP server.
///
/// # Panics
///
/// This method panics if:
///
///   * It fails to bind to the configured port.
///   * It fails to start the HTTP server.
pub fn spawn_server<A>(agent: A, context: AgentContext, upkeep: &mut Upkeep) -> Result<()>
where
    A: Agent + 'static,
{
    let agent: Arc<dyn Agent> = Arc::new(agent);
    let (send_server, receive_server) = sync_channel(0);
    let thread = Builder::new("r:b:api")
        .full_name("replicante:base:api")
        .spawn(move |scope| {
            let config = context.config.api.clone();
            let logger = context.logger.clone();
            let sentry_capture_api = context
                .config
                .sentry
                .as_ref()
                .map(|sentry| sentry.capture_api_errors.clone())
                .unwrap_or_default();

            // Initialise and configure HTTP server and App factory.
            let mut server = HttpServer::new(move || {
                let config = configure_app(Arc::clone(&agent), context.clone());
                // Give every mounted route access to the global context.
                let app = App::new().data(Arc::clone(&agent)).data(context.clone());
                // Register application middlewares.
                // Remember that middlewares are executed in reverse registration order.
                let app = app
                    .wrap(LoggingMiddleware::new(context.logger.clone()))
                    .wrap(MetricsMiddleware::new(REQUESTS.clone()))
                    .wrap(middleware::Compress::default());
                // Add the sentry middleware if configured.
                let app = match sentry_capture_api {
                    SentryCaptureApi::Client => app.wrap(SentryMiddleware::new(400)),
                    SentryCaptureApi::Server => app.wrap(SentryMiddleware::new(500)),
                    // acitx-web is so type safe that apps wrapped in middlewares change type.
                    // This means that even if we do not want to use the sentry middleware we need
                    // to configure it or we can't return a consisten type from this match.
                    SentryCaptureApi::No => app.wrap(SentryMiddleware::new(600)),
                };
                // Configure and return the ActixWeb App
                app.configure(config)
            })
            .keep_alive(config.timeouts.keep_alive);
            if let Some(read) = config.timeouts.read {
                server = server.client_timeout(read * 1000);
            }
            if let Some(write) = config.timeouts.write {
                server = server.client_shutdown(write * 1000);
            }
            if let Some(threads_count) = config.threads_count {
                server = server.workers(threads_count);
            }

            // Configure TLS/HTTPS if enabled and bind to the given address.
            let server = match config.tls {
                None => server
                    .bind(&config.bind)
                    .expect("unable to bind API server"),
                Some(tls) => {
                    let mut builder = SslAcceptor::mozilla_modern(SslMethod::tls())
                        .expect("unable to initialse TLS acceptor for API server");
                    builder
                        .set_certificate_file(&tls.server_cert, SslFiletype::PEM)
                        .expect("unable to set TLS server public certificate");
                    builder
                        .set_private_key_file(&tls.server_key, SslFiletype::PEM)
                        .expect("unable to set TLS server privte key");
                    if let Some(bundle) = tls.clients_ca_bundle {
                        builder
                            .set_ca_file(&bundle)
                            .expect("unable to set clients CAs bundle");
                        builder
                            .set_verify(SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT);
                    }
                    server
                        .bind_ssl(&config.bind, builder)
                        .expect("unable to bind API server")
                }
            };

            // Start HTTP server and block until shutdown.
            info!(logger, "Starting API server"; "bind" => &config.bind);
            scope.activity("running https://actix.rs/ HTTP(S) server");
            let runner = actix_rt::System::new("replicante:base:api");
            let system = actix_rt::System::current();
            send_server
                .send((server.start(), system))
                .expect("unable to send back server handle");
            runner.run().expect("unable to run API server");
        })
        .with_context(|_| ErrorKind::ThreadSpawn("api server"))?;
    upkeep.register_thread(thread);
    let (server, system) = receive_server
        .recv()
        .with_context(|_| ErrorKind::Initialisation("failed to spawn API server".into()))?;
    upkeep.on_shutdown(move || {
        let _ = server.stop(true).wait();
        system.stop();
    });
    Ok(())
}