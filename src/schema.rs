use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Schema {
    object_classes: Vec<ObjectClass>,
    attributes: Vec<Attribute>,
}

impl Schema {
    pub fn new() -> Schema {
        Schema {
            object_classes: Vec::new(),
            attributes: Vec::new(),
        }
    }

    pub fn validate_attributes(&self, attrs: HashMap<String, Vec<String>>) -> bool {
        // find object classes, ensure only one structural one

        // find all attrs for all classes (must/may)

        // check all must attrs exist and are valid (single/multi)

        // check any may attrs are valid (single multi)

        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct ObjectClass {
    numericoid: String,
    names: Vec<String>,
    desc: String,
    obsolete: bool,
    sup_oids: Vec<String>,
    kind: Kind,
    must_attrs: Vec<String>,
    may_attrs: Vec<String>,
}

#[derive(Debug, Copy, Clone)]
pub enum Kind {
    Abstract,
    Structural,
    Auxiliary,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub enum EqualityRule {}

#[derive(Debug, Copy, Clone)]
pub enum OrderingRule {}

#[derive(Debug, Copy, Clone)]
pub enum SubstringRule {}

#[derive(Debug, Copy, Clone)]
pub enum Usage {
    UserApplications,
    DirectoryOperations,
    DistributedOperation,
    DSAOperatoin,
}
