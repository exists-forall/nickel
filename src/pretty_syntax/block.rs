use pretty_trait::{Pretty, JoinExt, Join, Sep, Indent};

pub fn block<T: Pretty>(content: T) -> Join<Indent<Join<Sep, T>>, Sep> {
    Indent(Sep(0).join(content)).join(Sep(0))
}
