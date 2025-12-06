// Virtual DOM Implementation for UI Rendering
// Security: Layer 10 - API Security
// High-performance virtual DOM with diffing and patching

use std::collections::HashMap;
use std::fmt;

/// Virtual DOM Node types
#[derive(Debug, Clone, PartialEq)]
pub enum VNode {
    Element(VElement),
    Text(VText),
    Component(VComponent),
}

/// Virtual Element
#[derive(Debug, Clone, PartialEq)]
pub struct VElement {
    pub tag: String,
    pub props: HashMap<String, String>,
    pub children: Vec<VNode>,
    pub key: Option<String>,
}

/// Virtual Text Node
#[derive(Debug, Clone, PartialEq)]
pub struct VText {
    pub content: String,
}

/// Virtual Component
#[derive(Debug, Clone, PartialEq)]
pub struct VComponent {
    pub name: String,
    pub props: HashMap<String, String>,
    pub children: Vec<VNode>,
}

impl VNode {
    /// Create an element node
    pub fn element(tag: impl Into<String>) -> VElement {
        VElement {
            tag: tag.into(),
            props: HashMap::new(),
            children: Vec::new(),
            key: None,
        }
    }

    /// Create a text node
    pub fn text(content: impl Into<String>) -> VNode {
        VNode::Text(VText {
            content: content.into(),
        })
    }

    /// Create a component node
    pub fn component(name: impl Into<String>) -> VComponent {
        VComponent {
            name: name.into(),
            props: HashMap::new(),
            children: Vec::new(),
        }
    }
}

impl VElement {
    /// Add a property
    pub fn prop(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.props.insert(key.into(), value.into());
        self
    }

    /// Add a child node
    pub fn child(mut self, node: VNode) -> Self {
        self.children.push(node);
        self
    }

    /// Add multiple children
    pub fn children(mut self, nodes: Vec<VNode>) -> Self {
        self.children.extend(nodes);
        self
    }

    /// Set key for reconciliation
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Convert to VNode
    pub fn into_vnode(self) -> VNode {
        VNode::Element(self)
    }
}

impl VComponent {
    /// Add a property
    pub fn prop(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.props.insert(key.into(), value.into());
        self
    }

    /// Add a child node
    pub fn child(mut self, node: VNode) -> Self {
        self.children.push(node);
        self
    }

    /// Convert to VNode
    pub fn into_vnode(self) -> VNode {
        VNode::Component(self)
    }
}

/// Patch operations for updating the DOM
#[derive(Debug, Clone, PartialEq)]
pub enum Patch {
    /// Create a new node
    Create {
        path: Vec<usize>,
        node: VNode,
    },
    /// Remove a node
    Remove {
        path: Vec<usize>,
    },
    /// Replace a node
    Replace {
        path: Vec<usize>,
        node: VNode,
    },
    /// Update properties
    UpdateProps {
        path: Vec<usize>,
        props: HashMap<String, Option<String>>,
    },
    /// Update text content
    UpdateText {
        path: Vec<usize>,
        content: String,
    },
}

/// Virtual DOM implementation
pub struct VirtualDOM {
    root: Option<VNode>,
    previous_root: Option<VNode>,
}

impl VirtualDOM {
    /// Create a new Virtual DOM
    pub fn new() -> Self {
        Self {
            root: None,
            previous_root: None,
        }
    }

    /// Render a new tree
    pub fn render(&mut self, new_root: VNode) -> Vec<Patch> {
        let patches = if let Some(ref old_root) = self.root {
            self.diff(old_root, &new_root, vec![])
        } else {
            vec![Patch::Create {
                path: vec![],
                node: new_root.clone(),
            }]
        };

        self.previous_root = self.root.clone();
        self.root = Some(new_root);

        patches
    }

    /// Diff two virtual DOM trees
    fn diff(&self, old: &VNode, new: &VNode, path: Vec<usize>) -> Vec<Patch> {
        let mut patches = Vec::new();

        match (old, new) {
            (VNode::Element(old_el), VNode::Element(new_el)) => {
                if old_el.tag != new_el.tag {
                    // Different tag, replace entire node
                    patches.push(Patch::Replace {
                        path: path.clone(),
                        node: new.clone(),
                    });
                } else {
                    // Same tag, diff props and children
                    let prop_patches = self.diff_props(&old_el.props, &new_el.props, path.clone());
                    patches.extend(prop_patches);

                    let child_patches = self.diff_children(&old_el.children, &new_el.children, path);
                    patches.extend(child_patches);
                }
            }
            (VNode::Text(old_text), VNode::Text(new_text)) => {
                if old_text.content != new_text.content {
                    patches.push(Patch::UpdateText {
                        path,
                        content: new_text.content.clone(),
                    });
                }
            }
            (VNode::Component(old_comp), VNode::Component(new_comp)) => {
                if old_comp.name != new_comp.name {
                    patches.push(Patch::Replace {
                        path: path.clone(),
                        node: new.clone(),
                    });
                } else {
                    let prop_patches = self.diff_props(&old_comp.props, &new_comp.props, path.clone());
                    patches.extend(prop_patches);

                    let child_patches = self.diff_children(&old_comp.children, &new_comp.children, path);
                    patches.extend(child_patches);
                }
            }
            _ => {
                // Different node types, replace
                patches.push(Patch::Replace {
                    path,
                    node: new.clone(),
                });
            }
        }

        patches
    }

    /// Diff properties
    fn diff_props(
        &self,
        old_props: &HashMap<String, String>,
        new_props: &HashMap<String, String>,
        path: Vec<usize>,
    ) -> Vec<Patch> {
        let mut props_diff = HashMap::new();

        // Check for removed or changed props
        for (key, old_value) in old_props {
            if let Some(new_value) = new_props.get(key) {
                if old_value != new_value {
                    props_diff.insert(key.clone(), Some(new_value.clone()));
                }
            } else {
                props_diff.insert(key.clone(), None);
            }
        }

        // Check for added props
        for (key, new_value) in new_props {
            if !old_props.contains_key(key) {
                props_diff.insert(key.clone(), Some(new_value.clone()));
            }
        }

        if props_diff.is_empty() {
            vec![]
        } else {
            vec![Patch::UpdateProps {
                path,
                props: props_diff,
            }]
        }
    }

    /// Diff children
    fn diff_children(
        &self,
        old_children: &[VNode],
        new_children: &[VNode],
        mut path: Vec<usize>,
    ) -> Vec<Patch> {
        let mut patches = Vec::new();

        let old_len = old_children.len();
        let new_len = new_children.len();
        let min_len = old_len.min(new_len);

        // Diff existing children
        for i in 0..min_len {
            let mut child_path = path.clone();
            child_path.push(i);
            let child_patches = self.diff(&old_children[i], &new_children[i], child_path);
            patches.extend(child_patches);
        }

        // Handle added children
        if new_len > old_len {
            for i in old_len..new_len {
                let mut child_path = path.clone();
                child_path.push(i);
                patches.push(Patch::Create {
                    path: child_path,
                    node: new_children[i].clone(),
                });
            }
        }

        // Handle removed children
        if old_len > new_len {
            for i in (new_len..old_len).rev() {
                let mut child_path = path.clone();
                child_path.push(i);
                patches.push(Patch::Remove { path: child_path });
            }
        }

        patches
    }

    /// Get current root
    pub fn get_root(&self) -> Option<&VNode> {
        self.root.as_ref()
    }

    /// Convert VNode to HTML string (for SSR)
    pub fn to_html(&self) -> String {
        if let Some(ref root) = self.root {
            Self::node_to_html(root)
        } else {
            String::new()
        }
    }

    /// Convert a VNode to HTML string
    fn node_to_html(node: &VNode) -> String {
        match node {
            VNode::Element(el) => {
                let mut html = format!("<{}", el.tag);

                // Add properties
                for (key, value) in &el.props {
                    html.push_str(&format!(" {}=\"{}\"", key, Self::escape_html(value)));
                }

                html.push('>');

                // Add children
                for child in &el.children {
                    html.push_str(&Self::node_to_html(child));
                }

                html.push_str(&format!("</{}>", el.tag));
                html
            }
            VNode::Text(text) => Self::escape_html(&text.content),
            VNode::Component(comp) => {
                // Components are rendered as custom elements
                let mut html = format!("<{}", comp.name);

                for (key, value) in &comp.props {
                    html.push_str(&format!(" {}=\"{}\"", key, Self::escape_html(value)));
                }

                html.push('>');

                for child in &comp.children {
                    html.push_str(&Self::node_to_html(child));
                }

                html.push_str(&format!("</{}>", comp.name));
                html
            }
        }
    }

    /// Escape HTML special characters
    fn escape_html(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
}

impl Default for VirtualDOM {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for VNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", VirtualDOM::node_to_html(self))
    }
}

/// Helper macro for creating virtual DOM trees
#[macro_export]
macro_rules! vdom {
    // Element with no props or children
    ($tag:expr) => {
        VNode::element($tag).into_vnode()
    };

    // Element with props
    ($tag:expr, { $($key:expr => $value:expr),* }) => {{
        let mut el = VNode::element($tag);
        $(
            el = el.prop($key, $value);
        )*
        el.into_vnode()
    }};

    // Element with children
    ($tag:expr, [ $($child:expr),* ]) => {{
        let mut el = VNode::element($tag);
        $(
            el = el.child($child);
        )*
        el.into_vnode()
    }};

    // Element with props and children
    ($tag:expr, { $($key:expr => $value:expr),* }, [ $($child:expr),* ]) => {{
        let mut el = VNode::element($tag);
        $(
            el = el.prop($key, $value);
        )*
        $(
            el = el.child($child);
        )*
        el.into_vnode()
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vnode_creation() {
        let node = VNode::element("div")
            .prop("class", "container")
            .child(VNode::text("Hello, World!"))
            .into_vnode();

        match node {
            VNode::Element(el) => {
                assert_eq!(el.tag, "div");
                assert_eq!(el.props.get("class"), Some(&"container".to_string()));
                assert_eq!(el.children.len(), 1);
            }
            _ => panic!("Expected element node"),
        }
    }

    #[test]
    fn test_virtual_dom_render() {
        let mut vdom = VirtualDOM::new();

        let tree = VNode::element("div")
            .prop("id", "app")
            .child(VNode::text("Hello"))
            .into_vnode();

        let patches = vdom.render(tree);
        assert_eq!(patches.len(), 1);

        match &patches[0] {
            Patch::Create { path, .. } => {
                assert!(path.is_empty());
            }
            _ => panic!("Expected Create patch"),
        }
    }

    #[test]
    fn test_diff_same_tree() {
        let mut vdom = VirtualDOM::new();

        let tree = VNode::element("div")
            .child(VNode::text("Hello"))
            .into_vnode();

        vdom.render(tree.clone());
        let patches = vdom.render(tree);

        assert_eq!(patches.len(), 0);
    }

    #[test]
    fn test_diff_text_change() {
        let mut vdom = VirtualDOM::new();

        let tree1 = VNode::element("div")
            .child(VNode::text("Hello"))
            .into_vnode();

        let tree2 = VNode::element("div")
            .child(VNode::text("World"))
            .into_vnode();

        vdom.render(tree1);
        let patches = vdom.render(tree2);

        assert_eq!(patches.len(), 1);
        match &patches[0] {
            Patch::UpdateText { content, .. } => {
                assert_eq!(content, "World");
            }
            _ => panic!("Expected UpdateText patch"),
        }
    }

    #[test]
    fn test_diff_prop_change() {
        let mut vdom = VirtualDOM::new();

        let tree1 = VNode::element("div")
            .prop("class", "old")
            .into_vnode();

        let tree2 = VNode::element("div")
            .prop("class", "new")
            .into_vnode();

        vdom.render(tree1);
        let patches = vdom.render(tree2);

        assert_eq!(patches.len(), 1);
        match &patches[0] {
            Patch::UpdateProps { props, .. } => {
                assert_eq!(props.get("class"), Some(&Some("new".to_string())));
            }
            _ => panic!("Expected UpdateProps patch"),
        }
    }

    #[test]
    fn test_diff_add_child() {
        let mut vdom = VirtualDOM::new();

        let tree1 = VNode::element("div")
            .child(VNode::text("First"))
            .into_vnode();

        let tree2 = VNode::element("div")
            .child(VNode::text("First"))
            .child(VNode::text("Second"))
            .into_vnode();

        vdom.render(tree1);
        let patches = vdom.render(tree2);

        assert_eq!(patches.len(), 1);
        match &patches[0] {
            Patch::Create { path, .. } => {
                assert_eq!(path, &vec![1]);
            }
            _ => panic!("Expected Create patch"),
        }
    }

    #[test]
    fn test_diff_remove_child() {
        let mut vdom = VirtualDOM::new();

        let tree1 = VNode::element("div")
            .child(VNode::text("First"))
            .child(VNode::text("Second"))
            .into_vnode();

        let tree2 = VNode::element("div")
            .child(VNode::text("First"))
            .into_vnode();

        vdom.render(tree1);
        let patches = vdom.render(tree2);

        assert_eq!(patches.len(), 1);
        match &patches[0] {
            Patch::Remove { path } => {
                assert_eq!(path, &vec![1]);
            }
            _ => panic!("Expected Remove patch"),
        }
    }

    #[test]
    fn test_diff_replace_node() {
        let mut vdom = VirtualDOM::new();

        let tree1 = VNode::element("div")
            .child(VNode::text("Text"))
            .into_vnode();

        let tree2 = VNode::element("span")
            .child(VNode::text("Text"))
            .into_vnode();

        vdom.render(tree1);
        let patches = vdom.render(tree2);

        assert_eq!(patches.len(), 1);
        match &patches[0] {
            Patch::Replace { path, .. } => {
                assert!(path.is_empty());
            }
            _ => panic!("Expected Replace patch"),
        }
    }

    #[test]
    fn test_to_html() {
        let vdom = VirtualDOM::new();
        let mut vdom = vdom;

        let tree = VNode::element("div")
            .prop("class", "container")
            .prop("id", "app")
            .child(
                VNode::element("h1")
                    .child(VNode::text("Hello, World!"))
                    .into_vnode(),
            )
            .child(
                VNode::element("p")
                    .child(VNode::text("This is a paragraph."))
                    .into_vnode(),
            )
            .into_vnode();

        vdom.render(tree);
        let html = vdom.to_html();

        assert!(html.contains("<div"));
        assert!(html.contains("class=\"container\""));
        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello, World!"));
        assert!(html.contains("<p>"));
    }

    #[test]
    fn test_html_escaping() {
        let mut vdom = VirtualDOM::new();

        let tree = VNode::element("div")
            .child(VNode::text("<script>alert('xss')</script>"))
            .into_vnode();

        vdom.render(tree);
        let html = vdom.to_html();

        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_component_node() {
        let comp = VNode::component("Button")
            .prop("variant", "primary")
            .child(VNode::text("Click me"))
            .into_vnode();

        match comp {
            VNode::Component(c) => {
                assert_eq!(c.name, "Button");
                assert_eq!(c.props.get("variant"), Some(&"primary".to_string()));
                assert_eq!(c.children.len(), 1);
            }
            _ => panic!("Expected component node"),
        }
    }

    #[test]
    fn test_complex_tree() {
        let mut vdom = VirtualDOM::new();

        let tree = VNode::element("div")
            .prop("id", "root")
            .child(
                VNode::element("header")
                    .child(
                        VNode::element("nav")
                            .child(VNode::text("Navigation"))
                            .into_vnode(),
                    )
                    .into_vnode(),
            )
            .child(
                VNode::element("main")
                    .child(
                        VNode::element("article")
                            .child(VNode::text("Content"))
                            .into_vnode(),
                    )
                    .into_vnode(),
            )
            .child(
                VNode::element("footer")
                    .child(VNode::text("Footer"))
                    .into_vnode(),
            )
            .into_vnode();

        let patches = vdom.render(tree);
        assert_eq!(patches.len(), 1);

        let html = vdom.to_html();
        assert!(html.contains("<header>"));
        assert!(html.contains("<main>"));
        assert!(html.contains("<footer>"));
    }
}
