// use anyhow::bail;
// use kuchikiki::{ElementData, NodeDataRef, NodeRef};
// use markup5ever::{interface::QualName, namespace_url, ns, LocalName};

// pub trait NodeRefExt {
//     /// returns the `head` html element, if present  as child of `html`
//     fn head(&self) -> Option<NodeDataRef<ElementData>>;

//     fn inject_style(&self, sheet: &str)-> anyhow::Result<()>;
//     fn inject_script(&self, script: &str)-> anyhow::Result<()>;

//     /// finds first match
//     fn find_html_child_element(&self, name: &str) -> Option<NodeDataRef<ElementData>>;

//     /// Creates a new HTML element with the given name and attributes.
//     fn new_html_element(name: &str, attributes: Vec<(&str, &str)>) -> NodeRef;
// }
// impl NodeRefExt for NodeRef {

//     fn head(&self) -> Option<NodeDataRef<ElementData>>  {
//         let maybe_html: Option<_> = self.find_html_child_element("html");
//         if let Some(html) = maybe_html {
//             if let Some(head) = html.as_node().find_html_child_element("head") {
//                 return Some(head)
//             }
//         }
//         None
//     }

//     fn inject_style(&self, sheet: &str)-> anyhow::Result<()> {
//         match self.head() {
//             None => bail!("could not insert style, <head> element not found"),
//             Some(head) => {
//                 let style_node = NodeRef::new_html_element("style", vec![]);
//                 style_node.append(NodeRef::new_text(sheet));
//                 head.as_node().append(style_node);
//                 Ok(())
//             }
//         }
//     }

//     fn inject_script(&self, script: &str)-> anyhow::Result<()> {
//         match self.head() {
//             None => bail!("could not insert script, <head> element not found"),
//             Some(head) => {
//                 let script_node = NodeRef::new_html_element("style", vec![]);
//                 script_node.append(NodeRef::new_text(script));
//                 head.as_node().append(script_node);
//                 Ok(())
//             }
//         }
//     }




//     fn find_html_child_element(&self, name: &str) -> Option<NodeDataRef<ElementData>> {
//         let html_element_name: markup5ever::QualName = QualName::new(None, ns!(html),LocalName::from(name));
//         let maybe_head_node = self.children()
//         .find(|node| {
//             if let Some(element) = node.as_element() {
//                 if element.name == html_element_name {
//                     return true
//                 }
//             }
//             false
//         });
//         if let Some(head) = maybe_head_node {
//             return head.into_element_ref()
//         };
//         None
//     }


//     // thanks critter-rs for this utility function
//     fn new_html_element(name: &str, attributes: Vec<(&str, &str)>) -> NodeRef {
//         use kuchikiki::{Attribute, ExpandedName};

//         NodeRef::new_element(
//             QualName::new(None, ns!(html), LocalName::from(name)),
//             attributes.into_iter().map(|(n, v)| {
//                 (
//                     ExpandedName::new(ns!(), n),
//                     Attribute {
//                         prefix: None,
//                         value: v.to_string(),
//                     },
//                 )
//             }),
//         )
//     }
// }

// #[cfg(test)]
// mod tests {
//     use markup5ever::LocalName;

//     use super::*;

//     #[test]
//     fn test_head() {
//         use kuchikiki::traits::*;
//         let html: &str = r"
//             <!DOCTYPE html>
//             <html>
//             <head></head>
//             <body>
//                 <h1>Hello World!</h1>
//             </body>
//             </html>
//         ";

//         let document: NodeRef = kuchikiki::parse_html().one(html);
//         let e = document.head().unwrap();
//         assert_eq!(e.name.local, LocalName::from("head"));
//     }



//     #[test]
//     fn test_new_html_element_no_attrs() {
//         let node_ref = NodeRef::new_html_element("head", Vec::new());
//         assert!(node_ref.as_element().is_some());
//         let e = node_ref.as_element().unwrap();
//         assert_eq!(e.name.local, LocalName::from("head"));
//         assert!(e.attributes.borrow().map.is_empty());
//     }
// }