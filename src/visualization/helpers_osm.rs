use std::path::Path;

use osm4routing::{Edge, Node, NodeId};
use projection::Normalization;
use svg::{
    node::element::{self, Group},
    Document,
};

pub mod projection {
    // https://wiki.openstreetmap.org/wiki/Mercator#Rust

    use osm4routing::Node;

    // length of semi-major axis of the WGS84 ellipsoid, i.e. radius at equator
    const EARTH_RADIUS_KM: f64 = 6_378.137;

    pub fn lon2x(lon: f64) -> f64 {
        EARTH_RADIUS_KM * 1000. * lon.to_radians()
    }

    pub fn x2lon(x: f64) -> f64 {
        (x / (EARTH_RADIUS_KM * 1000.)).to_degrees()
    }

    pub fn lat2y(lat: f64) -> f64 {
        ((lat.to_radians() / 2. + std::f64::consts::PI / 4.).tan()).log(std::f64::consts::E)
            * EARTH_RADIUS_KM
            * 1000.
    }

    pub fn y2lat(y: f64) -> f64 {
        (2. * ((y / (EARTH_RADIUS_KM * 1000.)).exp()).atan() - std::f64::consts::PI / 2.)
            .to_degrees()
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Normalization {
        pub ratio: f64,
        pub x_min: f64,
        pub y_min: f64,
    }

    impl Normalization {
        pub fn for_nodes<'a, N>(nodes: N) -> Self
        where
            N: IntoIterator<Item = &'a Node>,
        {
            let mut x_min = f64::MAX;
            let mut x_max = f64::MIN;
            let mut y_min = f64::MAX;
            let mut y_max = f64::MIN;

            for node in nodes.into_iter() {
                x_min = x_min.min(lon2x(node.coord.x));
                x_max = x_max.max(lon2x(node.coord.x));
                y_min = y_min.min(lat2y(node.coord.y));
                y_max = y_max.max(lat2y(node.coord.y));
            }

            let x_size = x_max - x_min;
            let y_size = y_max - y_min;

            let ratio = (1.0 / x_size).min(1.0 / y_size);

            Self {
                ratio,
                x_min,
                y_min,
            }
        }

        pub fn normalize_lon(&self, lon: f64, scale: f64) -> f64 {
            (lon2x(lon) - self.x_min) * self.ratio * scale
        }

        pub fn normalize_lat(&self, lat: f64, scale: f64) -> f64 {
            scale - (lat2y(lat) - self.y_min) * self.ratio * scale
        }

        pub fn normalize(&self, coord: impl Into<(f64, f64)>, scale: f64) -> (f64, f64) {
            let (lon, lat) = coord.into();
            (
                self.normalize_lon(lon, scale),
                self.normalize_lat(lat, scale),
            )
        }
    }
}

pub struct SvgWriter<'a> {
    nodes: &'a [Node],
    edges: &'a [Edge],
    simplify_edges: bool,
    normalization: Option<Normalization>,
    scale: f64,
    node_classes: Option<&'a dyn Fn(NodeId) -> Option<String>>,
    edge_classes: Option<&'a dyn Fn((NodeId, NodeId)) -> Option<String>>,
    extra_style: Option<String>,
}

impl<'a> SvgWriter<'a> {
    pub fn for_graph(nodes: &'a [Node], edges: &'a [Edge]) -> Self {
        Self {
            normalization: None,
            scale: 1.0,
            nodes,
            edges,
            simplify_edges: true,
            node_classes: None,
            edge_classes: None,
            extra_style: None,
        }
    }

    pub fn simplify_edges(mut self, simplify: bool) -> Self {
        self.simplify_edges = simplify;
        self
    }

    pub fn with_normalization(mut self, normalization: Normalization) -> Self {
        self.normalization = Some(normalization);
        self
    }

    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_edge_classes(
        mut self,
        edge_classes: &'a dyn Fn((NodeId, NodeId)) -> Option<String>,
    ) -> Self {
        self.edge_classes = Some(edge_classes);
        self
    }

    pub fn with_extra_style(mut self, extra_style: String) -> Self {
        self.extra_style = Some(extra_style);
        self
    }

    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let normalization = self
            .normalization
            .unwrap_or_else(|| Normalization::for_nodes(self.nodes));

        let mut document = Document::new().set("viewBox", (0, 0, self.scale, self.scale));

        document = document.add(element::Style::new(format!(
            ".edges path {{ fill: none; stroke: black; stroke-width: 0.5; }}\n\
            .nodes circle {{ fill: black }}\n\
            {}",
            self.extra_style.as_ref().unwrap_or(&"".to_string())
        )));

        let mut node_group = Group::new().set("class", "nodes");
        for node in self.nodes {
            let (x, y) = normalization.normalize(node.coord, self.scale);
            let mut circle = element::Circle::new()
                .set("cx", x)
                .set("cy", y)
                .set("r", "1");
            if let Some(node_classes) = &self.node_classes {
                if let Some(classes) = node_classes(node.id) {
                    circle = circle.set("class", classes);
                }
            }
            node_group = node_group.add(circle);
        }
        document = document.add(node_group);

        let mut edge_group = Group::new().set("class", "edges");
        for edge in self.edges {
            let data = if self.simplify_edges {
                element::path::Data::new()
                    .move_to(normalization.normalize(*edge.geometry.first().unwrap(), self.scale))
                    .line_to(normalization.normalize(*edge.geometry.last().unwrap(), self.scale))
            } else {
                let mut data = element::path::Data::new()
                    .move_to(normalization.normalize(edge.geometry[0], self.scale));
                for coord in edge.geometry.iter().skip(1) {
                    data = data.line_to(normalization.normalize(*coord, self.scale));
                }
                data
            };
            let mut path = element::Path::new().set("d", data);

            if let Some(edge_classes) = &self.edge_classes {
                if let Some(classes) = edge_classes((edge.source, edge.target)) {
                    path = path.set("class", classes);
                }
            }
            edge_group = edge_group.add(path);
        }
        document = document.add(edge_group);

        svg::save(path, &document)
    }
}
