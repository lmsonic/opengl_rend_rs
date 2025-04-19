use std::{collections::HashMap, fs::File, io::BufReader, path::Path, str::FromStr};

use gl::types::{GLbyte, GLfloat, GLint, GLshort, GLsizeiptr, GLubyte, GLuint, GLushort};
use glam::bool;
use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use crate::{
    buffer::{Buffer, BufferType, Usage},
    opengl::{IndexSize, OpenGl, Primitive},
    vertex_attributes::{DataType, VertexArrayObject, VertexAttribute},
};

#[derive(Debug, PartialEq, Clone, Copy)]
enum AttributeData {
    Float(GLfloat),
    UnsignedInt(GLuint),
    Int(GLint),
    UnsignedShort(GLushort),
    Short(GLshort),
    UnsignedByte(GLubyte),
    Byte(GLbyte),
}

fn parse_index_data(index_size: IndexSize, s: &str) -> Option<AttributeData> {
    match index_size {
        IndexSize::UnsignedByte => Some(AttributeData::UnsignedByte(s.parse::<GLubyte>().ok()?)),
        IndexSize::UnsignedShort => Some(AttributeData::UnsignedShort(s.parse::<GLushort>().ok()?)),
        IndexSize::UnsignedInt => Some(AttributeData::UnsignedInt(s.parse::<GLuint>().ok()?)),
    }
}

fn parse_data(data_type: DataType, s: &str) -> Option<AttributeData> {
    match data_type {
        DataType::Byte => Some(AttributeData::Byte(s.parse::<GLbyte>().ok()?)),
        DataType::UnsignedByte => Some(AttributeData::UnsignedByte(s.parse::<GLubyte>().ok()?)),
        DataType::Short => Some(AttributeData::UnsignedShort(s.parse::<GLushort>().ok()?)),
        DataType::UnsignedShort => Some(AttributeData::Short(s.parse::<GLshort>().ok()?)),
        DataType::Int => Some(AttributeData::Int(s.parse::<GLint>().ok()?)),
        DataType::UnsignedInt => Some(AttributeData::UnsignedInt(s.parse::<GLuint>().ok()?)),
        DataType::Float => Some(AttributeData::Float(s.parse::<GLfloat>().ok()?)),
        DataType::Double | DataType::Fixed => unimplemented!(),
    }
}

struct Attribute {
    index: GLuint,
    vertex_attribute: VertexAttribute,
    data: Vec<AttributeData>,
}

fn parse_data_type(s: &str) -> Option<(DataType, bool)> {
    match s {
        "float" => Some((DataType::Float, false)),
        "half" => Some((DataType::Fixed, false)),
        "int" => Some((DataType::Int, false)),
        "uint" => Some((DataType::UnsignedInt, false)),
        "norm-int" => Some((DataType::Int, true)),
        "norm-uint" => Some((DataType::UnsignedInt, true)),
        "short" => Some((DataType::Short, false)),
        "ushort" => Some((DataType::UnsignedShort, false)),
        "norm-short" => Some((DataType::Short, true)),
        "norm-ushort" => Some((DataType::UnsignedShort, true)),
        "byte" => Some((DataType::Byte, false)),
        "ubyte" => Some((DataType::UnsignedByte, false)),
        "norm-byte" => Some((DataType::Byte, true)),
        "norm-ubyte" => Some((DataType::UnsignedByte, true)),
        _ => None,
    }
}

fn find_attribute(attributes: &[OwnedAttribute], name: &str) -> Option<String> {
    attributes
        .iter()
        .find(|a| a.name.local_name == name)
        .map(|a| a.value.clone())
}

fn find_attribute_unwrap(attributes: &[OwnedAttribute], name: &str) -> String {
    find_attribute(attributes, name)
        .unwrap_or_else(|| panic!("Unable to find attribute with name {name}"))
}

fn find_attribute_parse_unwrap<T: FromStr>(attributes: &[OwnedAttribute], name: &str) -> T {
    find_attribute_unwrap(attributes, name)
        .parse::<T>()
        .unwrap_or_else(|_| {
            panic!(
                "Unable to parse attribute with name {name} to {}",
                stringify!(T)
            )
        })
}

impl Attribute {
    fn new(attributes: &[OwnedAttribute], string_data: &str) -> Self {
        let index = find_attribute_parse_unwrap::<GLuint>(attributes, "index");
        assert!(index <= 16, "Attribute index must be between 0 and 16.");

        let size = find_attribute_parse_unwrap::<GLint>(attributes, "size");
        assert!(size >= 1, "Attribute size must be between 1 and 5.");
        assert!(size <= 5, "Attribute size must be between 1 and 5.");

        let data_type = find_attribute_unwrap(attributes, "type");
        let (data_type, normalized) = parse_data_type(&data_type).expect("Unknown 'type' field.");

        let integral = find_attribute(attributes, "integral");
        if let Some(integral) = integral {
            let is_integral = integral
                .parse::<bool>()
                .expect("Incorrect 'integral' value for the 'attribute'.");
            if normalized && is_integral {
                panic!("cannot be both integral and normalized");
            }
            if data_type.is_floating_point() && is_integral {
                panic!("cannot be both integral and floating point");
            }
        }
        let vertex_attribute = VertexAttribute::new(size, data_type, normalized);
        // parse data
        let mut data = vec![];
        for word in string_data.split_whitespace() {
            let value = parse_data(data_type, word).expect("Parse error in array data stream.");
            data.push(value);
        }
        Self {
            index,
            vertex_attribute,
            data,
        }
    }

    fn num_elements(&self) -> usize {
        self.data.len() * self.vertex_attribute.components as usize
    }
    fn byte_size(&self) -> usize {
        self.data.len() * self.vertex_attribute.data_type.size()
    }

    fn setup_attribute_array(&self, vao: &mut VertexArrayObject, offset: GLint) {
        vao.set_attribute(self.index, &self.vertex_attribute, 0, offset);
    }
}

fn process_vao(
    vao_attributes: &[OwnedAttribute],
    source_attributes: &[OwnedAttribute],
) -> (String, Vec<GLuint>) {
    let name = find_attribute_unwrap(vao_attributes, "name");
    let mut attributes = vec![];
    for attrib in source_attributes {
        assert_eq!(attrib.name.local_name, "attrib");
        let value = attrib
            .value
            .parse::<GLuint>()
            .expect("VAO Attribute parse error");
        attributes.push(value);
    }
    (name, attributes)
}

struct IndicesData {
    data_type: IndexSize,
    data: Vec<AttributeData>,
}

fn parse_index_type(s: &str) -> Option<(IndexSize, bool)> {
    match s {
        "uint" => Some((IndexSize::UnsignedInt, false)),
        "norm-uint" => Some((IndexSize::UnsignedInt, true)),
        "ushort" => Some((IndexSize::UnsignedShort, false)),
        "norm-ushort" => Some((IndexSize::UnsignedShort, true)),
        "ubyte" => Some((IndexSize::UnsignedByte, false)),
        "norm-ubyte" => Some((IndexSize::UnsignedByte, true)),
        _ => None,
    }
}

impl IndicesData {
    fn new(attributes: &[OwnedAttribute], string_data: &str) -> Self {
        let data_type = find_attribute_unwrap(attributes, "type");
        let (data_type, _) = parse_index_type(&data_type)
            .expect("Improper 'type' attribute value on 'index' element.");

        // parse data
        let mut data = vec![];
        for word in string_data.split_whitespace() {
            let value =
                parse_index_data(data_type, word).expect("Parse error in array data stream.");
            data.push(value);
        }
        Self { data_type, data }
    }

    fn byte_size(&self) -> usize {
        self.data.len() * self.data_type.size()
    }
}
#[derive(Clone, Copy)]
enum RenderCommand {
    Indexed {
        primitive: Primitive,
        count: GLint,
        index_size: IndexSize,
        offset: usize,
        primitive_restart: Option<GLuint>,
    },
    Array {
        primitive: Primitive,
        start: GLint,
        end: GLint,
    },
}

fn parse_primitive(s: &str) -> Option<Primitive> {
    match s {
        "lines" => Some(Primitive::Lines),
        "triangles" => Some(Primitive::Triangles),
        "tri-strip" => Some(Primitive::TriangleStrip),
        "tri-fan" => Some(Primitive::TriangleFan),
        "line-strip" => Some(Primitive::LineStrip),
        "line-loop" => Some(Primitive::LineLoop),
        "points" => Some(Primitive::Points),
        _ => None,
    }
}

impl RenderCommand {
    fn new(name: &str, attributes: &[OwnedAttribute]) -> Self {
        let primitive = find_attribute_unwrap(attributes, "cmd");
        let primitive = parse_primitive(&primitive).expect("Unknown 'cmd' field.");
        match name {
            "indices" => {
                let primitive_restart = find_attribute(attributes, "prim-restart")
                    .and_then(|s| s.parse::<GLuint>().ok());
                // count, index size, and offset will filled out lated
                RenderCommand::Indexed {
                    primitive,
                    primitive_restart,
                    count: 0,
                    index_size: IndexSize::UnsignedInt,
                    offset: 0,
                }
            }
            "arrays" => {
                let start = find_attribute_parse_unwrap::<GLint>(attributes, "start");
                assert!(
                    start >= 0,
                    "`array` 'start' index must be between 0 or greater."
                );
                let end = find_attribute_parse_unwrap::<GLint>(attributes, "end");
                assert!(end > 0, "`array` 'count' must be between 0 or greater.");
                RenderCommand::Array {
                    primitive,
                    start,
                    end,
                }
            }
            _ => panic!("Bad command element {name} Must be 'indices' or 'arrays'."),
        }
    }

    fn render(self, gl: &mut OpenGl) {
        match self {
            RenderCommand::Indexed {
                primitive,
                count,
                index_size,
                offset,
                ..
            } => gl.draw_elements(primitive, count, index_size, offset),
            RenderCommand::Array {
                primitive,
                start,
                end,
            } => gl.draw_arrays(primitive, start, end),
        }
    }
}

struct MeshData {
    attrib_array_buffer: Buffer<AttributeData>,
    index_buffer: Buffer<AttributeData>,
    vao: VertexArrayObject,
    named_vaos: HashMap<String, VertexArrayObject>,
    commands: Vec<RenderCommand>,
}

impl MeshData {
    fn new() -> Self {
        Self {
            attrib_array_buffer: Buffer::new(BufferType::ArrayBuffer),
            index_buffer: Buffer::new(BufferType::IndexBuffer),
            vao: VertexArrayObject::new(),
            named_vaos: HashMap::new(),
            commands: Vec::new(),
        }
    }
}

pub struct Mesh {
    mesh_data: MeshData,
}

struct ParsedData {
    attribs: Vec<Attribute>,
    indices_list: Vec<IndicesData>,
    named_vao_list: Vec<(String, Vec<GLuint>)>,
    commands: Vec<RenderCommand>,
}

impl Mesh {
    fn parse_xml(path: impl AsRef<Path>) -> ParsedData {
        let mut attribs: Vec<Attribute> = Vec::with_capacity(16);
        // Map from Attribute indices to the indices in the attribs vector just created [0,16]
        let mut indices_list: Vec<IndicesData> = vec![];
        let mut named_vao_list: Vec<(String, Vec<GLuint>)> = vec![];
        let mut commands: Vec<RenderCommand> = vec![];

        let path = path.as_ref();
        let file = File::open(path).unwrap_or_else(|_| panic!("Unable to open file {path:?}"));
        let file = BufReader::new(file);
        #[derive(PartialEq, Eq)]
        enum ParserState {
            Initial,
            JustPassedMeshRoot,
            InAttributeTag {
                attributes: Vec<OwnedAttribute>,
            },
            InVaoTag {
                vao_attributes: Vec<OwnedAttribute>,
                sources_attributes: Vec<OwnedAttribute>,
            },
            InIndicesTag {
                attributes: Vec<OwnedAttribute>,
            },
        }

        let mut parser_state = ParserState::Initial;

        let parser = EventReader::new(file);
        let mut depth = 0;
        for e in parser {
            match e {
                Ok(event) => match event {
                    XmlEvent::EndDocument => break,
                    XmlEvent::StartElement {
                        name,
                        mut attributes,
                        ..
                    } => {
                        match depth {
                            0 => {
                                if name.local_name != "mesh" {
                                    panic!("`mesh` node not found in mesh file: {path:?}")
                                }
                                parser_state = ParserState::JustPassedMeshRoot;
                            }
                            1 => {
                                let name = name.local_name;
                                if parser_state == ParserState::JustPassedMeshRoot
                                    && name != "attribute"
                                {
                                    panic!("`mesh` node must have at least one `attribute` child. File: {path:?}")
                                } else {
                                    parser_state = ParserState::Initial;
                                }
                                match name.as_str() {
                                    "attribute" => {
                                        dbg!("attribute");
                                        parser_state = ParserState::InAttributeTag { attributes };
                                    }
                                    "vao" => {
                                        dbg!("vao");
                                        parser_state = ParserState::InVaoTag {
                                            vao_attributes: attributes,
                                            sources_attributes: vec![],
                                        };
                                    }

                                    _ => {
                                        // assumes either arrays or indices i guess?
                                        dbg!("indices");
                                        let primitive = RenderCommand::new(&name, &attributes);
                                        if let RenderCommand::Indexed { .. } = primitive {
                                            parser_state = ParserState::InIndicesTag { attributes };
                                        }
                                        commands.push(primitive);
                                    }
                                }
                            }
                            2 => {
                                if name.local_name == "source" {
                                    dbg!("source");
                                    match parser_state {
                                        ParserState::InVaoTag {
                                            ref mut sources_attributes,
                                            ..
                                        } => {
                                            sources_attributes.append(&mut attributes);
                                        }
                                        _ => panic!("source tag is only valid when in vao tag"),
                                    }
                                } else {
                                    panic!("this depth only allowed for source tags")
                                }
                            }
                            _ => {}
                        }
                        depth += 1;
                    }
                    XmlEvent::Characters(data) => match parser_state {
                        ParserState::InAttributeTag { attributes } => {
                            dbg!("attribute data");
                            let attribute = Attribute::new(&attributes, &data);
                            attribs.push(attribute);
                            parser_state = ParserState::Initial;
                        }
                        ParserState::InIndicesTag { attributes } => {
                            dbg!("indices data");
                            let data = IndicesData::new(&attributes, &data);
                            indices_list.push(data);
                            parser_state = ParserState::Initial;
                        }
                        _ => {}
                    },
                    XmlEvent::EndElement { .. } => {
                        // HACK: EndElement will trigger when source tag ends so we have to check for depth :\
                        if depth <= 2 {
                            if let ParserState::InVaoTag {
                                vao_attributes,
                                sources_attributes,
                            } = parser_state
                            {
                                dbg!("vao end");
                                // can only do this at the end
                                let (name, vaos) =
                                    process_vao(&vao_attributes, &sources_attributes);
                                named_vao_list.push((name, vaos));
                                dbg!(&named_vao_list);
                                parser_state = ParserState::Initial;
                            };
                        }

                        depth -= 1;
                    }
                    _ => {}
                },
                Err(err) => eprintln!("Error : {err}"),
            }
        }
        ParsedData {
            attribs,
            indices_list,
            named_vao_list,
            commands,
        }
    }

    pub fn new(path: impl AsRef<Path>) -> Self {
        let parsed_data = Self::parse_xml(path);

        let mut mesh_data = MeshData::new();
        mesh_data.commands = parsed_data.commands;
        // this is trying to calculate how much they need to allocate for attributes
        let mut attribute_buffer_size = 0;
        let mut attribute_start_locs = Vec::with_capacity(parsed_data.attribs.len());
        let mut num_elements = 0;
        for attrib in &parsed_data.attribs {
            attribute_buffer_size = if attribute_buffer_size % 16 != 0 {
                // i hate the c++ code i took this from. WTF
                // i guess it might be alignment?
                attribute_buffer_size + (16 - attribute_buffer_size % 16)
            } else {
                attribute_buffer_size
            };
            attribute_start_locs.push(attribute_buffer_size);

            attribute_buffer_size += attrib.byte_size();

            if num_elements != 0 {
                assert_eq!(
                    num_elements,
                    attrib.num_elements(),
                    "Some of the attribute arrays have different element counts."
                )
            } else {
                num_elements = attrib.num_elements();
            }
        }

        mesh_data.vao.bind();
        mesh_data.attrib_array_buffer.bind();
        mesh_data
            .attrib_array_buffer
            .reserve_data(attribute_buffer_size as GLsizeiptr, Usage::StaticDraw);

        for (i, attrib) in parsed_data.attribs.iter().enumerate() {
            let offset = attribute_start_locs[i];
            mesh_data
                .attrib_array_buffer
                .update_data(&attrib.data, offset as isize);
            attrib.setup_attribute_array(&mut mesh_data.vao, offset as GLint);
        }

        // fill named vaos
        for (name, source_list) in parsed_data.named_vao_list {
            let mut vao = VertexArrayObject::new();
            vao.bind();
            for attrib in source_list {
                let offset = parsed_data
                    .attribs
                    .iter()
                    .position(|a| a.index == attrib)
                    .unwrap_or_else(|| {
                        panic!("could not find source index {attrib} for vao {name}")
                    });

                parsed_data.attribs[offset].setup_attribute_array(&mut vao, offset as GLint);
            }
            mesh_data.named_vaos.insert(name, vao);
        }
        mesh_data.vao.unbind();

        // calculate index buffer size
        let mut index_buffer_size = 0;
        let mut index_start_locs = Vec::with_capacity(parsed_data.indices_list.len());
        for data in &parsed_data.indices_list {
            index_buffer_size = if index_buffer_size % 16 != 0 {
                index_buffer_size + (16 - index_buffer_size % 16)
            } else {
                index_buffer_size
            };
            index_start_locs.push(index_buffer_size);
            index_buffer_size += data.byte_size();
        }

        // create index buffer
        if index_buffer_size > 0 {
            mesh_data.vao.bind();
            mesh_data.index_buffer.bind();
            mesh_data
                .index_buffer
                .reserve_data(index_buffer_size as GLsizeiptr, Usage::StaticDraw);

            // fill in data
            for (i, data) in parsed_data.indices_list.iter().enumerate() {
                let offset = index_start_locs[i];
                mesh_data
                    .index_buffer
                    .update_data(&data.data, offset as isize);
            }
            // fill in indexed rendering commands like said earlier
            // TODO: possibly merge commands and indices in a render pass struct or something like that
            // How to deal with arrays commands thou?
            let mut i = 0;
            for commands in &mut mesh_data.commands {
                if let RenderCommand::Indexed {
                    ref mut count,
                    ref mut index_size,
                    ref mut offset,
                    ..
                } = commands
                {
                    *offset = index_start_locs[i];
                    *count = parsed_data.indices_list[i].data.len() as GLint;
                    *index_size = parsed_data.indices_list[i].data_type;
                    i += 1;
                }
            }

            for vao in mesh_data.named_vaos.values_mut() {
                vao.bind();
                mesh_data.index_buffer.bind();
            }
            mesh_data.vao.unbind();
        }

        Self { mesh_data }
    }
    pub fn render(&mut self, gl: &mut OpenGl) {
        self.mesh_data.vao.bind();
        for cmd in &self.mesh_data.commands {
            cmd.render(gl);
        }
        self.mesh_data.vao.unbind();
    }
    pub fn render_mesh(&mut self, mesh_name: String, gl: &mut OpenGl) {
        let Some((_, vao)) = self
            .mesh_data
            .named_vaos
            .iter_mut()
            .find(|(name, _)| **name == mesh_name)
        else {
            return;
        };

        vao.bind();
        for cmd in &self.mesh_data.commands {
            cmd.render(gl);
        }
        vao.unbind();
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use gl::types::{GLuint, GLushort};

    use crate::{
        mesh::{AttributeData, RenderCommand},
        opengl::{IndexSize, Primitive},
        vertex_attributes::{DataType, VertexAttribute},
    };

    use super::{Attribute, IndicesData, Mesh};
    macro_rules! test_case {
        ($fname:expr) => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/resources/test/", $fname) // assumes Linux ('/')!
        };
    }

    fn test_attribute(
        attribute: &Attribute,
        index: GLuint,
        vertex_attribute: VertexAttribute,
        data: &[AttributeData],
    ) {
        assert_eq!(attribute.index, index);
        assert_eq!(
            attribute.vertex_attribute.data_type,
            vertex_attribute.data_type
        );
        assert_eq!(
            attribute.vertex_attribute.components,
            vertex_attribute.components
        );
        assert_eq!(
            attribute.vertex_attribute.normalized,
            vertex_attribute.normalized
        );
        // testing attribute data
        assert_eq!(attribute.data.len(), data.len());
        if attribute.data.is_empty() {
            return;
        }
        let first = attribute.data[0];
        if let AttributeData::Float(_) = first {
            // assume all values are float
            for (i, attribute) in attribute.data.iter().enumerate() {
                let a = match attribute {
                    AttributeData::Float(a) => a,
                    _ => panic!(),
                };
                let b = match data[i] {
                    AttributeData::Float(b) => b,
                    _ => panic!(),
                };

                assert!((a - b).abs() < f32::EPSILON, "{i}: {a} {b}");
            }
        } else {
            // assume all values are integral
            assert_eq!(attribute.data, data)
        }
    }

    fn test_indices(indices: &IndicesData, index_size: IndexSize, data: &[AttributeData]) {
        assert_eq!(indices.data_type, index_size);
        assert_eq!(indices.data, data);
    }
    fn test_named_vaos(named_vaos: &[(String, Vec<u32>)], expected: &[(&str, Vec<u32>)]) {
        assert_eq!(named_vaos.len(), expected.len());
        for (i, (name, vao)) in named_vaos.iter().enumerate() {
            assert_eq!(name, expected[i].0);
            assert_eq!(*vao, expected[i].1);
        }
    }

    fn test_commands(cmd: &RenderCommand, expected_primitive: Primitive) {
        if let RenderCommand::Indexed { primitive, .. } = cmd {
            assert_eq!(*primitive, expected_primitive);
            // all the other things are not known until the other calculations
        } else {
            panic!();
        };
    }
    #[test]
    fn test_plane_parse() {
        let file_path = Path::new(test_case!("UnitPlane.xml"));

        let parsed_xml = Mesh::parse_xml(file_path);

        // testing attributes
        assert_eq!(parsed_xml.attribs.len(), 1);
        let attribute = &parsed_xml.attribs[0];
        let data = [
            0.5, 0.0, -0.5, 0.5, 0.0, 0.5, -0.5, 0.0, 0.5, -0.5, 0.0, -0.5,
        ]
        .map(AttributeData::Float);

        test_attribute(
            attribute,
            0,
            VertexAttribute::new(3, DataType::Float, false),
            &data,
        );

        // testing indices
        assert_eq!(parsed_xml.indices_list.len(), 1);
        let indices = &parsed_xml.indices_list[0];

        let data = [0, 1, 2, 0, 2, 1, 2, 3, 0, 2, 0, 3]
            .iter()
            .map(|n: &GLushort| AttributeData::UnsignedShort(*n))
            .collect::<Vec<_>>();

        test_indices(indices, IndexSize::UnsignedShort, &data);

        assert_eq!(parsed_xml.named_vao_list.len(), 0);

        assert_eq!(parsed_xml.commands.len(), 1);
        let cmd = &parsed_xml.commands[0];
        test_commands(cmd, Primitive::Triangles);
    }

    #[test]
    fn test_cube_parse() {
        let file_path = Path::new(test_case!("UnitCube.xml"));

        let parsed_xml = Mesh::parse_xml(file_path);

        // testing attributes
        assert_eq!(parsed_xml.attribs.len(), 1);
        let attribute = &parsed_xml.attribs[0];

        // testing attribute data
        let data = [
            0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5,
            0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5,
            0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
        ]
        .map(AttributeData::Float);
        test_attribute(
            attribute,
            0,
            VertexAttribute::new(3, DataType::Float, false),
            &data,
        );

        // testing indices
        assert_eq!(parsed_xml.indices_list.len(), 1);
        let indices = &parsed_xml.indices_list[0];
        assert_eq!(indices.data_type, IndexSize::UnsignedShort);
        let data: Vec<AttributeData> = [
            0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16,
            17, 18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
        ]
        .iter()
        .map(|n: &GLushort| AttributeData::UnsignedShort(*n))
        .collect();
        test_indices(indices, IndexSize::UnsignedShort, &data);

        assert_eq!(parsed_xml.named_vao_list.len(), 0);

        assert_eq!(parsed_xml.commands.len(), 1);
        let cmd = &parsed_xml.commands[0];
        test_commands(cmd, Primitive::Triangles);
    }

    #[test]
    fn test_cone_parse() {
        let file_path = Path::new(test_case!("UnitCone.xml"));

        let parsed_xml = Mesh::parse_xml(file_path);

        // testing attributes
        assert_eq!(parsed_xml.attribs.len(), 1);
        let attribute = &parsed_xml.attribs[0];

        // testing attribute data
        #[allow(clippy::excessive_precision)]
        let data = [
            0.0,
            0.866,
            0.0,
            0.5,
            0.0,
            0.0,
            0.48907381875731,
            0.0,
            0.1039557588888,
            0.45677280077542,
            0.0,
            0.20336815992623,
            0.40450865316151,
            0.0,
            0.29389241146627,
            0.33456556611288,
            0.0,
            0.37157217599218,
            0.2500003830126,
            0.0,
            0.43301248075957,
            0.15450900193016,
            0.0,
            0.47552809414644,
            0.052264847412855,
            0.0,
            0.49726088296277,
            -0.052263527886268,
            0.0,
            0.49726102165048,
            -0.15450774007312,
            0.0,
            0.47552850414828,
            -0.24999923397422,
            0.0,
            0.43301314415651,
            -0.33456458011157,
            0.0,
            0.37157306379065,
            -0.40450787329018,
            0.0,
            0.29389348486527,
            -0.45677226111814,
            0.0,
            0.20336937201315,
            -0.48907354289964,
            0.0,
            0.10395705668972,
            -0.49999999999824,
            0.0,
            1.3267948966764e-006,
            -0.48907409461153,
            0.0,
            -0.10395446108714,
            -0.45677334042948,
            0.0,
            -0.20336694783787,
            -0.40450943302999,
            0.0,
            -0.2938913380652,
            -0.33456655211184,
            0.0,
            -0.3715712881911,
            -0.25000153204922,
            0.0,
            -0.43301181735958,
            -0.15451026378611,
            0.0,
            -0.47552768414126,
            -0.052266166939075,
            0.0,
            -0.49726074427155,
            0.052262208359312,
            0.0,
            -0.4972611603347,
            0.15450647821499,
            0.0,
            -0.47552891414676,
            0.24999808493408,
            0.0,
            -0.4330138075504,
            0.3345635941079,
            0.0,
            -0.37157395158649,
            0.40450709341601,
            0.0,
            -0.2938945582622,
            0.45677172145764,
            0.0,
            -0.20337058409865,
            0.48907326703854,
            0.0,
            -0.10395835448992,
            0.0,
            0.0,
            0.0,
        ]
        .map(AttributeData::Float);

        test_attribute(
            attribute,
            0,
            VertexAttribute::new(3, DataType::Float, false),
            &data,
        );
        // testing indices
        assert_eq!(parsed_xml.indices_list.len(), 2);
        let indices = &parsed_xml.indices_list[0];
        assert_eq!(indices.data_type, IndexSize::UnsignedShort);
        let data: Vec<AttributeData> = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 1,
        ]
        .iter()
        .map(|n: &GLushort| AttributeData::UnsignedShort(*n))
        .collect();
        test_indices(indices, IndexSize::UnsignedShort, &data);

        let indices = &parsed_xml.indices_list[1];
        let data: Vec<AttributeData> = [
            31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10,
            9, 8, 7, 6, 5, 4, 3, 2, 1, 30,
        ]
        .iter()
        .map(|n: &GLushort| AttributeData::UnsignedShort(*n))
        .collect();
        test_indices(indices, IndexSize::UnsignedShort, &data);

        assert_eq!(parsed_xml.named_vao_list.len(), 0);

        assert_eq!(parsed_xml.commands.len(), 2);
        let cmd = &parsed_xml.commands[0];
        test_commands(cmd, Primitive::TriangleFan);
        let cmd = &parsed_xml.commands[1];
        test_commands(cmd, Primitive::TriangleFan);
    }

    #[test]
    fn test_cube_color_parse() {
        let file_path = Path::new(test_case!("UnitCubeColor.xml"));

        let parsed_xml = Mesh::parse_xml(file_path);

        // testing attributes
        assert_eq!(parsed_xml.attribs.len(), 2);
        let attribute = &parsed_xml.attribs[0];

        let data = [
            0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5,
            0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5,
            0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
        ]
        .map(AttributeData::Float);
        test_attribute(
            attribute,
            0,
            VertexAttribute::new(3, DataType::Float, false),
            &data,
        );

        let attribute = &parsed_xml.attribs[1];
        let data = [
            0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0,
            0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0,
            0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0,
            1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0,
            0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0,
            0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0,
        ]
        .map(AttributeData::Float);
        test_attribute(
            attribute,
            1,
            VertexAttribute::new(4, DataType::Float, false),
            &data,
        );

        // testing indices
        assert_eq!(parsed_xml.indices_list.len(), 1);
        let indices = &parsed_xml.indices_list[0];
        let data: Vec<AttributeData> = [
            0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16,
            17, 18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
        ]
        .iter()
        .map(|n: &GLushort| AttributeData::UnsignedShort(*n))
        .collect();
        test_indices(indices, IndexSize::UnsignedShort, &data);

        assert_eq!(parsed_xml.named_vao_list.len(), 0);

        assert_eq!(parsed_xml.commands.len(), 1);
        let cmd = &parsed_xml.commands[0];
        test_commands(cmd, Primitive::Triangles);
    }

    #[test]
    fn test_cone_color_parse() {
        let file_path = Path::new(test_case!("UnitConeTint.xml"));

        let parsed_xml = Mesh::parse_xml(file_path);

        // testing attributes
        assert_eq!(parsed_xml.attribs.len(), 2);
        let attribute = &parsed_xml.attribs[0];

        // testing attribute data
        #[allow(clippy::excessive_precision)]
        let data = [
            0.0,
            0.866,
            0.0,
            0.5,
            0.0,
            0.0,
            0.48907381875731,
            0.0,
            0.1039557588888,
            0.45677280077542,
            0.0,
            0.20336815992623,
            0.40450865316151,
            0.0,
            0.29389241146627,
            0.33456556611288,
            0.0,
            0.37157217599218,
            0.2500003830126,
            0.0,
            0.43301248075957,
            0.15450900193016,
            0.0,
            0.47552809414644,
            0.052264847412855,
            0.0,
            0.49726088296277,
            -0.052263527886268,
            0.0,
            0.49726102165048,
            -0.15450774007312,
            0.0,
            0.47552850414828,
            -0.24999923397422,
            0.0,
            0.43301314415651,
            -0.33456458011157,
            0.0,
            0.37157306379065,
            -0.40450787329018,
            0.0,
            0.29389348486527,
            -0.45677226111814,
            0.0,
            0.20336937201315,
            -0.48907354289964,
            0.0,
            0.10395705668972,
            -0.49999999999824,
            0.0,
            1.3267948966764e-006,
            -0.48907409461153,
            0.0,
            -0.10395446108714,
            -0.45677334042948,
            0.0,
            -0.20336694783787,
            -0.40450943302999,
            0.0,
            -0.2938913380652,
            -0.33456655211184,
            0.0,
            -0.3715712881911,
            -0.25000153204922,
            0.0,
            -0.43301181735958,
            -0.15451026378611,
            0.0,
            -0.47552768414126,
            -0.052266166939075,
            0.0,
            -0.49726074427155,
            0.052262208359312,
            0.0,
            -0.4972611603347,
            0.15450647821499,
            0.0,
            -0.47552891414676,
            0.24999808493408,
            0.0,
            -0.4330138075504,
            0.3345635941079,
            0.0,
            -0.37157395158649,
            0.40450709341601,
            0.0,
            -0.2938945582622,
            0.45677172145764,
            0.0,
            -0.20337058409865,
            0.48907326703854,
            0.0,
            -0.10395835448992,
            0.0,
            0.0,
            0.0,
        ]
        .map(AttributeData::Float);

        test_attribute(
            attribute,
            0,
            VertexAttribute::new(3, DataType::Float, false),
            &data,
        );
        let attribute = &parsed_xml.attribs[1];
        let data = [
            1.0, 1.0, 1.0, 1.0, 0.9, 0.9, 0.9, 1.0, 0.82, 0.82, 0.82, 1.0, 0.74, 0.74, 0.74, 1.0,
            0.66, 0.66, 0.66, 1.0, 0.58, 0.58, 0.58, 1.0, 0.5, 0.5, 0.5, 1.0, 0.58, 0.58, 0.58,
            1.0, 0.66, 0.66, 0.66, 1.0, 0.74, 0.74, 0.74, 1.0, 0.82, 0.82, 0.82, 1.0, 0.9, 0.9,
            0.9, 1.0, 0.82, 0.82, 0.82, 1.0, 0.74, 0.74, 0.74, 1.0, 0.66, 0.66, 0.66, 1.0, 0.58,
            0.58, 0.58, 1.0, 0.5, 0.5, 0.5, 1.0, 0.58, 0.58, 0.58, 1.0, 0.66, 0.66, 0.66, 1.0,
            0.74, 0.74, 0.74, 1.0, 0.82, 0.82, 0.82, 1.0, 0.9, 0.9, 0.9, 1.0, 0.82, 0.82, 0.82,
            1.0, 0.74, 0.74, 0.74, 1.0, 0.66, 0.66, 0.66, 1.0, 0.58, 0.58, 0.58, 1.0, 0.5, 0.5,
            0.5, 1.0, 0.58, 0.58, 0.58, 1.0, 0.66, 0.66, 0.66, 1.0, 0.74, 0.74, 0.74, 1.0, 0.82,
            0.82, 0.82, 1.0, 0.9, 0.9, 0.9, 1.0,
        ]
        .map(AttributeData::Float);
        test_attribute(
            attribute,
            1,
            VertexAttribute::new(4, DataType::Float, false),
            &data,
        );
        // testing indices
        assert_eq!(parsed_xml.indices_list.len(), 2);
        let indices = &parsed_xml.indices_list[0];
        assert_eq!(indices.data_type, IndexSize::UnsignedShort);
        let data: Vec<AttributeData> = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 1,
        ]
        .iter()
        .map(|n: &GLushort| AttributeData::UnsignedShort(*n))
        .collect();
        test_indices(indices, IndexSize::UnsignedShort, &data);

        let indices = &parsed_xml.indices_list[1];
        let data: Vec<AttributeData> = [
            31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10,
            9, 8, 7, 6, 5, 4, 3, 2, 1, 30,
        ]
        .iter()
        .map(|n: &GLushort| AttributeData::UnsignedShort(*n))
        .collect();
        test_indices(indices, IndexSize::UnsignedShort, &data);

        assert_eq!(parsed_xml.named_vao_list.len(), 0);

        assert_eq!(parsed_xml.commands.len(), 2);
        let cmd = &parsed_xml.commands[0];
        test_commands(cmd, Primitive::TriangleFan);
        let cmd = &parsed_xml.commands[1];
        test_commands(cmd, Primitive::TriangleFan);
    }
    #[test]
    fn test_cube_tint_parse() {
        let file_path = Path::new(test_case!("UnitCubeTint.xml"));

        let parsed_xml = Mesh::parse_xml(file_path);

        // testing attributes
        assert_eq!(parsed_xml.attribs.len(), 2);
        let attribute = &parsed_xml.attribs[0];

        let data = [
            0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5,
            0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5,
            0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
        ]
        .map(AttributeData::Float);
        test_attribute(
            attribute,
            0,
            VertexAttribute::new(3, DataType::Float, false),
            &data,
        );

        let attribute = &parsed_xml.attribs[1];
        let data = [
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.75,
            0.75, 0.75, 1.0, 0.75, 0.75, 0.75, 1.0, 0.75, 0.75, 0.75, 1.0, 0.75, 0.75, 0.75, 1.0,
            0.5, 0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 1.0, 1.0,
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.75, 0.75,
            0.75, 1.0, 0.75, 0.75, 0.75, 1.0, 0.75, 0.75, 0.75, 1.0, 0.75, 0.75, 0.75, 1.0, 0.5,
            0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 1.0,
        ]
        .map(AttributeData::Float);
        test_attribute(
            attribute,
            1,
            VertexAttribute::new(4, DataType::Float, false),
            &data,
        );

        // testing indices
        assert_eq!(parsed_xml.indices_list.len(), 1);
        let indices = &parsed_xml.indices_list[0];
        let data: Vec<AttributeData> = [
            0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16,
            17, 18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
        ]
        .iter()
        .map(|n: &GLushort| AttributeData::UnsignedShort(*n))
        .collect();
        test_indices(indices, IndexSize::UnsignedShort, &data);

        assert_eq!(parsed_xml.named_vao_list.len(), 0);

        assert_eq!(parsed_xml.commands.len(), 1);
        let cmd = &parsed_xml.commands[0];
        test_commands(cmd, Primitive::Triangles);
    }
    #[test]
    fn test_sphere_vao() {
        let file_path = Path::new(test_case!("UnitSphere.xml"));

        let parsed_xml = Mesh::parse_xml(file_path);
        let expected = [
            ("lit-color", vec![0, 1, 2]),
            ("lit", vec![0, 2]),
            ("color", vec![0, 1]),
            ("flat", vec![0]),
        ];
        test_named_vaos(&parsed_xml.named_vao_list, &expected);
    }
}
