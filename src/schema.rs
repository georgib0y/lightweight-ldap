use std::collections::HashMap;

type OID = String;
type NOIDLen = String;
type Extensions = HashMap<String, Vec<String>>;

enum Kind {
    Abstract,
    Structural,
    Auxiliary,
}

struct ObjectClassSchema {
    oid: OID,
    names: Vec<String>,
    desc: Option<String>,
    obsolete: bool,
    sups: Vec<OID>,
    kind: Option<Kind>,
    required_atts: Vec<OID>,
    optional_atts: Vec<OID>,
    extensions: Extensions,
}

enum Usage {
    UserApplications,
    DirectoryOperation,
    DistributedOperation,
    DSAOperation,
}

struct AttributeSchema {
    oid: OID,
    names: Vec<String>,
    desc: Option<String>,
    obsolete: bool,
    sups: Option<OID>,
    equality: Option<OID>,
    ordering: Option<OID>,
    substr: Option<OID>,
    syntax: Option<NOIDLen>,
    single_value: bool,
    collective: bool,
    modifiable: bool,
    usage: Option<Usage>,
    extensions: Extensions,
}
