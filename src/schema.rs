pub struct Schema {
    object_classes: Vec<ObjectClass>,
    attributes: Vec<Attribute>
}

pub struct ObjectClass {
    numericoid: String,
    names: Vec<String>,
    desc: String,
    obsolete: bool,
    sup_oids: Vec<String>,
    kind: Kind,
    must_attrs: Vec<String>,
    may_attrs: Vec<String>
}

pub enum Kind {
    Abstract,
    Structural,
    Auxiliary
}

pub struct Attribute {
    numericoid: String,
    name: String,
    desc: String,
    obsolete: bool,
    sup_oids: Vec<String>,
    equality_rule: EqualityRule,
    ordering_rule: OrderingRule,
    substr_rule: SubstringRule,
    syntax: String,
    single_value: bool,
    collective: bool,
    no_user_modification: bool,
    usage: Usage,
    extensions: String,
}

pub enum EqualityRule {}
pub enum OrderingRule {}
pub enum SubstringRule {}

pub enum Usage {
    UserApplications,
    DirectoryOperations,
    DistributedOperation,
    DSAOperatoin
}
