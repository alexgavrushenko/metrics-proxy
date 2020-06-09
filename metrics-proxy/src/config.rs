use clap::{App, Arg};
use std::convert::TryFrom;
use std::net::SocketAddr;

/// Discribes service configuration.
#[derive(Clone, Debug)]
pub struct MetricsServiceConfig {
    /// Name of the service.
    pub name: String,
}

impl TryFrom<String> for MetricsServiceConfig {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let v: Vec<_> = s.split('@').collect();
        if v.len() == 2 {
            Ok(MetricsServiceConfig {
                name: v[0].to_owned(),
            })
        } else {
            Err(format!("Failed to parse MetricsServiceConfig from {}", s))
        }
    }
}

/// Defines the proxy config as services.
#[derive(Clone, Debug)]
pub struct ProxyServiceConfig {
    /// Http address that serve connection for service.
    pub listener_addr: SocketAddr,
    /// List of services to communicate with
    pub services: Vec<MetricsServiceConfig>,
}

pub struct ProxyAppConfig {
    pub service_config: ProxyServiceConfig,
}

/// Get a proxy configuration struct from command line arguments or releated environment
/// variables.
/// IMPORTANT: Sentry and logging options will be read from env: `SENTRY_DSN` for sentry,
/// `LOG_JSON` to log in Json format.
///
/// USAGE:
///    app_name [OPTIONS] --http-listener-addr <http-listener-addr> --services <services>
///
/// FLAGS:
///    -h, --help       Prints help information
///    -V, --version    Prints version information
///
/// OPTIONS:
///    -l, --http-listener-addr <http-listener-addr>
///           HTTP address for service to listen to [env: PROXY_HTTP_LISTENER_ADDR=]
///
///    -s, --services <services>...                     
///           List of services to communicate with in format `service_name1@addr:port,service_name2@addr:port` [env:
///           PROXY_SERVICES=]

pub fn from_args_proxy_config(app: App<'_, '_>) -> ProxyAppConfig {
    let env_http_listener_addr = "PROXY_HTTP_LISTENER_ADDR";
    let env_services = "PROXY_SERVICES";

    let arguments = app
        .arg(
            Arg::with_name("http-listener-addr")
                .short("l")
                .long("http-listener-addr")
                .required(true)
                .takes_value(true)
                .env(&env_http_listener_addr)
                .help("HTTP address for service to listen to"),
        )
        .arg(
            Arg::with_name("services")
                .short("s")
                .long("services")
                .required(true)
                .takes_value(true)
                .env(&env_services)
                .help("List of services to communicate with in format `service_name1@addr:port,service_name2@addr:port`"),
        ).get_matches();

    let listener_addr = arguments
        .value_of("http-listener-addr")
        .unwrap()
        .parse()
        .unwrap();
    let services = arguments
        .value_of("services")
        .unwrap()
        .split(',')
        .map(|s| s.split(' '))
        .flatten()
        .map(|s| s.to_owned())
        .filter(|s| !s.is_empty())
        .map(|s| MetricsServiceConfig::try_from(s).unwrap())
        .collect::<Vec<_>>();

    ProxyAppConfig {
        service_config: ProxyServiceConfig {
            listener_addr,
            services,
        },
    }
}
