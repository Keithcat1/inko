//! Various test helper functions and types.
use crate::hir;
use crate::state::State;
use location::Location;
use types::module_name::ModuleName;
use types::{
    Module, ModuleId, Symbol, Trait, TypeRef, Visibility, DROP_MODULE,
    DROP_TRAIT,
};

pub(crate) fn loc(
    line_start: u32,
    line_end: u32,
    column_start: u32,
    column_end: u32,
) -> Location {
    Location { line_start, line_end, column_start, column_end }
}

pub(crate) fn cols(start: u32, stop: u32) -> Location {
    Location {
        line_start: 1,
        line_end: 1,
        column_start: start,
        column_end: stop,
    }
}

pub(crate) fn hir_module(
    state: &mut State,
    name: ModuleName,
    expressions: Vec<hir::TopLevelExpression>,
) -> hir::Module {
    hir::Module {
        documentation: String::new(),
        module_id: Module::alloc(&mut state.db, name, "test.inko".into()),
        expressions,
        location: cols(1, 1),
    }
}

pub(crate) fn hir_type_name(
    name: &str,
    arguments: Vec<hir::Type>,
    location: Location,
) -> hir::TypeName {
    hir::TypeName {
        self_type: false,
        source: None,
        resolved_type: TypeRef::Unknown,
        name: hir::Constant { name: name.to_string(), location },
        arguments,
        location,
    }
}

pub(crate) fn module_type(state: &mut State, name: &str) -> ModuleId {
    Module::alloc(
        &mut state.db,
        ModuleName::new(name),
        format!("{}.inko", name).into(),
    )
}

pub(crate) fn define_drop_trait(state: &mut State) {
    let module = Module::alloc(
        &mut state.db,
        ModuleName::new(DROP_MODULE),
        "drop.inko".into(),
    );

    let drop_trait = Trait::alloc(
        &mut state.db,
        DROP_TRAIT.to_string(),
        Visibility::Public,
        module,
        Location::default(),
    );

    module.new_symbol(
        &mut state.db,
        DROP_TRAIT.to_string(),
        Symbol::Trait(drop_trait),
    );
}
