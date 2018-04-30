use std::rc::Rc;

pub fn rc_str(s: &str) -> Rc<String> {
    Rc::new(s.to_owned())
}
