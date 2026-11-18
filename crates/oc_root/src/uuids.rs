use derive_more::Constructor;
use uuid::Uuid;

macro_rules! uuid_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[oc_derive::archivable(bare)]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord,
            serde::Serialize, serde::Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// New random uuid
            pub fn new_v4() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(s).map(Self)
            }
        }
    };
}

uuid_type! { SquadUuid }
uuid_type! { IndividualUuid }

#[oc_derive::archivable(bare)]
#[derive(Debug, Clone, Default, PartialEq, Constructor)]
pub struct Uuids {
    /// Indexed like world individuals
    pub individuals: Vec<IndividualUuid>,
    /// Indexed like world squads (can be empty when world is not built from deployments)
    pub squads: Vec<SquadUuid>,
}

impl Uuids {
    pub fn individual(&self, i: usize) -> &IndividualUuid {
        &self.individuals[i]
    }

    pub fn squad(&self, i: usize) -> Option<&SquadUuid> {
        self.squads.get(i)
    }
}
