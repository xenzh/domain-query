#![feature(associated_type_defaults)]

use domain_query::{domain, value};
use std::fmt::{Display, Formatter, Result as FmtResult};
use strum_macros::{EnumIter, EnumString};

#[derive(PartialEq, Clone, Copy, Hash, Eq, Debug, EnumIter, EnumString)]
enum Property {
    #[strum(serialize="AlbumName", serialize="albumname", serialize="album_name")]
    AlbumName,
    AlbumArtist,
    AlbumReleaseDate,
    AlbumListeners,
    AlbumPlayCount,
    AlbumTracks,
    TrackName,
}

impl Display for Property {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "whatever")
    }
}

impl domain::DomainEnum for Property {}

impl domain::Property for Property {
    fn name(&self) -> &'static str {
        "property"
    }

    fn datatype(&self) -> value::Datatype {
        value::Datatype::Int
    }
}

#[derive(PartialEq, Clone, Copy, Hash, Eq, Debug, EnumIter, EnumString)]
enum Entity {
    Album,
    Track,
}

impl Entity {
    const PROPS: &'static [Property] = &[
        Property::AlbumName,
        Property::AlbumArtist,
        Property::AlbumReleaseDate,
    ];
}

impl Display for Entity {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "whatever")
    }
}

impl domain::DomainEnum for Entity {}

impl domain::Entity<Property> for Entity {
    fn name(&self) -> &str {
        "entity"
    }

    fn properties(&self) -> &[Property] {
        Entity::PROPS
    }
}

//type Music = domain::Domain<Property, Entity>;

//#[test]
//fn domain_property() {
//    let alname = Music::property("AlbumName");
//    assert!(alname.is_ok());
//    assert_eq!(alname.unwrap(), Property::AlbumName);
//}

//#[test]
//fn domain_property_notfound() {
//    let notf = Music::property("Notfound");
//    assert!(notf.is_err());
//    match notf.err().unwrap() {
//        error::Error::IdentifierNotFound(_) => {},
//        _ => panic!("Unexpected error type"),
//    };
//}

//#[test]
//fn domain_entity() {
//    let track = Music::entity("Track");
//    assert!(track.is_ok());
//    assert_eq!(track.unwrap(), Entity::Track);
//}

//#[test]
//fn domain_entity_notfound() {
//    let notf = Music::entity("Notfound");
//    assert!(notf.is_err());
//    match notf.err().unwrap() {
//        error::Error::IdentifierNotFound(_) => {},
//        _ => panic!("Unexpected error type"),
//    };
//}

