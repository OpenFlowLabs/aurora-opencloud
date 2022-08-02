pub mod definition;

mod tests {

    #[test]
    fn it_works() {
    use pretty_assertions::{assert_eq};
    use std::fs;
    use miette::{IntoDiagnostic, Context};
    use std::collections::HashMap;
    use crate::definition::{Document, Action, Volume, Ips, IpsPublisher, CaCertificates, Mediator, VolumeProperty};

    let file = "testdata/image_base.kdl";

    let comparision = Document{ 
        author: Some("John Doe <john.doe@example.com>".into()), 
        name: "my-image".into(), 
        version: 1, 
        base_on: Some("img://openindiana.org/hipster".into()), 
        actions: vec![
            Action::Volume(Volume{ 
                name: "data".into(), 
                mountpoint: Some("/var/lib/pgdata".into()), 
                properties: vec![
                    VolumeProperty {
                        name: "checksum".into(),
                        value: "off".into(),
                        driver_name: "zfs".into(),
                    },
                    VolumeProperty {
                        name: "compression".into(),
                        value: "lz4".into(),
                        driver_name: "zfs".into(),
                    },
                    VolumeProperty {
                        name: "copies".into(),
                        value: "3".into(),
                        driver_name: "zfs".into(),
                    },
                    VolumeProperty {
                        name: "bar".into(),
                        value: "1".into(),
                        driver_name: "foo".into(),
                    },
                ] 
            }),
            Action::Ips(Ips{ 
                packages: vec![
                    "developer/gcc-11".into(),
                    "golang".into(),
                    "golang-118".into(),
                ], 
                install_optionals: true, 
                properties: vec![
                    HashMap::from([("image.prop".into(), "false".into())])
                ], 
                publishers: vec![IpsPublisher{ 
                    publisher: "openindiana.org".into(), 
                    uris: vec!["https://pkg.openindiana.org/hipster".into()]
                },], 
                ca_certificates: vec![
                    CaCertificates{ 
                        publisher: "openindiana.org".into(), 
                        cert_file: "/path/to/cert/in/image/bundle".into() 
                    }
                ], 
                uninstall: vec![
                    "userland-incorportation".into()
                ], 
                variants: vec![
                    HashMap::from([("opensolaris.zone".into(), "global".into())])
                ],
                facets: vec![
                    HashMap::from([("my.facet.name".into(), "true".into())])
                ], 
                purge_history: true, 
                mediators: vec![
                    Mediator{ 
                        name: "mysql".into(), 
                        implementation: Some("mariadb".into()), 
                        version: None 
                    }
                ] 
            })
        ], 
    };

    let text = fs::read_to_string(file).into_diagnostic()
        .wrap_err_with(|| format!("cannot read {:?}", file)).unwrap();

    let config = match knuffel::parse::<Document>(file, &text) {
        Ok(config) => config,
        Err(e) => {
            panic!("{:?}", miette::Report::new(e));
        }
    };

    assert_eq!(comparision, config);

    }
}