use super::TestSerialization;
use crate::html5::html5ever::Dom;
use html5ever::{namespace_url, ns};
use markup5ever_rcdom::{Handle, NodeData};

// Adapted from https://github.com/servo/html5ever/blob/8415d500150d3232036bd2fb9681e7820fd7ecea/rcdom/tests/html-tree-builder.rs#L77
fn serialize(writer: &mut String, indent: usize, handle: Handle) {
    let mut buf = String::new();

    let node = handle;
    match node.data {
        NodeData::Document => {}

        NodeData::Doctype {
            ref name,
            ref public_id,
            ref system_id,
        } => {
            buf.push_str("<!DOCTYPE ");
            buf.push_str(name);
            if !public_id.is_empty() || !system_id.is_empty() {
                buf.push_str(&format!(" \"{public_id}\" \"{system_id}\""));
            }
            buf.push_str(">\n");
        }

        NodeData::Text { ref contents } => {
            let text = &contents.borrow();
            let text = text.strip_suffix("\n").unwrap_or(text);
            if !text.is_empty() {
                buf.push('"');
                buf.push_str(text);
                buf.push_str("\"\n");
            }
        }

        NodeData::Comment { ref contents } => {
            buf.push_str("<!-- ");
            buf.push_str(contents);
            buf.push_str(" -->\n");
        }

        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            buf.push('<');
            match name.ns {
                ns!(svg) => buf.push_str("svg "),
                ns!(mathml) => buf.push_str("math "),
                _ => {}
            }
            buf.push_str(&name.local);
            buf.push_str(">\n");

            let mut attrs = attrs.borrow().clone();
            attrs.sort_by(|x, y| x.name.local.cmp(&y.name.local));
            // FIXME: sort by UTF-16 code unit

            for attr in attrs.into_iter() {
                buf.push('|');
                buf.extend(std::iter::repeat(" ").take(indent + 2));
                match attr.name.ns {
                    ns!(xlink) => buf.push_str("xlink "),
                    ns!(xml) => buf.push_str("xml "),
                    ns!(xmlns) => buf.push_str("xmlns "),
                    _ => {}
                }
                buf.push_str(&format!("{}=\"{}\"\n", attr.name.local, attr.value));
            }
        }

        NodeData::ProcessingInstruction { .. } => unreachable!(),
    }

    for child in node.children.borrow().iter() {
        serialize(&mut buf, indent + 2, child.clone());
    }

    if let NodeData::Element {
        ref template_contents,
        ..
    } = node.data
    {
        if let Some(ref content) = &*template_contents.borrow() {
            buf.push('|');
            buf.extend(std::iter::repeat(" ").take(indent + 2));
            buf.push_str("content\n");
            for child in content.children.borrow().iter() {
                serialize(&mut buf, indent + 4, child.clone());
            }
        }
    }

    if !buf.is_empty() {
        writer.push('|');
        writer.extend(std::iter::repeat(" ").take(indent));
        writer.push_str(&buf);
    }
}

impl TestSerialization for Dom {
    fn serialize(&mut self) -> String {
        let document: Handle = self.0.document.clone();
        let children = document.children.borrow();
        let node = children.as_slice().first().unwrap();
        let mut buf = String::new();
        serialize(&mut buf, 1, node.clone());
        buf
    }
}
