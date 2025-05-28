// See: <https://doc.rust-lang.org/rustdoc/unstable-features.html#extensions-to-the-doc-attribute>
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg, doc_cfg_hide),
    doc(cfg_hide(doc))
)]

#[cfg(feature = "unstable")]
pub mod from_py_dict;
pub mod py_match;
pub mod py_wrapper;
pub mod ungil;
