use std::{collections::HashMap, fs::File, io::BufReader, path::Path, str::FromStr};

use gl::types::{GLbyte, GLenum, GLfloat, GLint, GLintptr, GLshort, GLubyte, GLuint, GLushort};
use glam::bool;
use xml::{attribute::OwnedAttribute, name::OwnedName, reader::XmlEvent, EventReader};

use crate::{
    buffer::{Buffer, BufferType},
    opengl::{IndexSize, OpenGl, Primitive},
    program::GLLocation,
    vertex_attributes::{DataType, VertexArrayObject, VertexAttribute},
};

enum AttributeData {
    Float(GLfloat),
    UnsignedInt(GLuint),
    Int(GLint),
    UnsignedShort(GLushort),
    Short(GLshort),
    UnsignedByte(GLubyte),
    Byte(GLbyte),
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

    fn fill_bound_buffer_object(&self, gl: &mut OpenGl, offset: GLintptr) {
        // i dont like this
        unsafe {
            gl::BufferSubData(
                BufferType::IndexBuffer as GLenum,
                offset,
                std::mem::size_of_val(&self.data) as isize,
                self.data.as_ptr() as *const _,
            )
        };
    }
    fn setup_attribute_array(&self, gl: &mut OpenGl, vao: &mut VertexArrayObject, offset: GLint) {
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

struct IndexData {
    data_type: DataType,
    data: Vec<AttributeData>,
}

impl IndexData {
    fn new(attributes: &[OwnedAttribute], string_data: &str) -> Self {
        let data_type = find_attribute_unwrap(attributes, "type");
        let (data_type, _) = parse_data_type(&data_type)
            .expect("Improper 'type' attribute value on 'index' element.");
        assert_eq!(
            data_type,
            DataType::UnsignedByte,
            "Improper 'type' attribute value on 'index' element."
        );
        assert_eq!(
            data_type,
            DataType::UnsignedInt,
            "Improper 'type' attribute value on 'index' element."
        );
        assert_eq!(
            data_type,
            DataType::UnsignedShort,
            "Improper 'type' attribute value on 'index' element."
        );

        // parse data
        let mut data = vec![];
        for word in string_data.split_whitespace() {
            let value = parse_data(data_type, word).expect("Parse error in array data stream.");
            data.push(value);
        }
        Self { data_type, data }
    }

    fn byte_size(&self) -> usize {
        self.data.len() * self.data_type.size()
    }

    fn fill_bound_buffer_object(&self, gl: &mut OpenGl, offset: GLintptr) {
        // i dont like this
        unsafe {
            gl::BufferSubData(
                BufferType::IndexBuffer as GLenum,
                offset,
                std::mem::size_of_val(&self.data) as isize,
                self.data.as_ptr() as *const _,
            )
        };
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
                    count: 0,
                    index_size: IndexSize::UnsignedInt,
                    offset: 0,
                    primitive_restart,
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
    attributes_array_buffer: Buffer<Attribute>,
    index_buffer: Buffer<GLuint>,
    vao: VertexArrayObject,
    named_vaos: HashMap<String, VertexArrayObject>,
    primitives: Vec<RenderCommand>,
}

impl MeshData {
    fn new() -> Self {
        Self {
            attributes_array_buffer: Buffer::new(BufferType::ArrayBuffer),
            index_buffer: Buffer::new(BufferType::IndexBuffer),
            vao: VertexArrayObject::new(),
            named_vaos: HashMap::new(),
            primitives: Vec::new(),
        }
    }
}

pub struct Mesh {
    mesh_data: MeshData,
}

impl Mesh {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let mut mesh_data = MeshData::new();
        let mut attribs: Vec<Attribute> = Vec::with_capacity(16);
        // Map from Attribute indices to the indices in the attribs vector just created [0,16]
        let mut attrib_index_map: HashMap<GLuint, usize> = HashMap::new();
        let mut index_data: Vec<IndexData> = vec![];
        let mut named_vao_list: Vec<(String, Vec<GLuint>)> = vec![];

        let path = path.as_ref();
        let file = File::open(path).unwrap_or_else(|_| panic!("Unable to open file {path:?}"));
        let file = BufReader::new(file);
        #[derive(PartialEq, Eq)]
        enum ParserState {
            Initial,
            JustPassedMeshRoot,
            InAttributeTag { attributes: Vec<OwnedAttribute> },
            InVaoTag { attributes: Vec<OwnedAttribute> },
            InIndicesTag { attributes: Vec<OwnedAttribute> },
        }

        let mut parser_state = ParserState::Initial;

        let parser = EventReader::new(file);
        let mut depth = 0;
        for e in parser {
            match e {
                Ok(event) => match event {
                    XmlEvent::EndDocument => break,
                    XmlEvent::StartElement {
                        name, attributes, ..
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
                                        parser_state = ParserState::InAttributeTag { attributes };
                                    }
                                    "vao" => {
                                        parser_state = ParserState::InVaoTag { attributes };
                                    }

                                    _ => {
                                        // assumes either arrays or indices i guess?
                                        let primitive = RenderCommand::new(&name, &attributes);
                                        if let RenderCommand::Indexed { .. } = primitive {
                                            parser_state = ParserState::InIndicesTag { attributes };
                                        }
                                        mesh_data.primitives.push(primitive);
                                    }
                                }
                            }
                            2 => {
                                if name.local_name == "source" {
                                    match parser_state {
                                        ParserState::InVaoTag {
                                            attributes: vao_attributes,
                                        } => {
                                            // can only do it if we have both!
                                            let (name, source_attribs) =
                                                process_vao(&vao_attributes, &attributes);
                                            named_vao_list.push((name, source_attribs));
                                            parser_state = ParserState::Initial;
                                        }
                                        _ => panic!(
                                            "source tag is only valid as a child of the vao tag"
                                        ),
                                    }
                                }
                            }
                            _ => {}
                        }
                        depth += 1;
                    }
                    XmlEvent::Characters(data) => match parser_state {
                        ParserState::InAttributeTag { attributes } => {
                            let attribute = Attribute::new(&attributes, &data);
                            let index = attribute.index;
                            attribs.push(attribute);
                            attrib_index_map.insert(index, attribs.len() - 1);
                            parser_state = ParserState::Initial;
                        }
                        ParserState::InIndicesTag { attributes } => {
                            let data = IndexData::new(&attributes, &data);
                            index_data.push(data);
                            parser_state = ParserState::Initial;
                        }
                        _ => {}
                    },
                    XmlEvent::EndElement { name } => {
                        depth -= 1;
                    }
                    _ => {}
                },
                Err(err) => eprintln!("Error : {err}"),
            }
        }
        Self { mesh_data }
    }
    pub fn render(&mut self) {}
    pub fn render_mesh(&mut self, mesh_name: String) {}
    pub fn delete_objects(&mut self) {}
}
