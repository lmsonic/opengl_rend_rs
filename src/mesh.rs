use std::{io::Read, path::Path};

use gl::types::{GLbyte, GLenum, GLfloat, GLint, GLintptr, GLshort, GLubyte, GLuint, GLushort};
use glam::bool;
use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use crate::{
    buffer::{Buffer, BufferType},
    opengl::{IndexSize, OpenGl, Primitive},
    vertex_attributes::{DataType, VertexArrayObject, VertexAttribute},
};

#[derive(Clone, Copy)]
enum RenderCommand {
    Indexed {
        primitive: Primitive,
        count: GLint,
        index_size: IndexSize,
        offset: usize,
    },
    Normal {
        primitive: Primitive,
        start: GLint,
        end: GLint,
    },
}

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

impl Attribute {
    fn new(attributes: &[OwnedAttribute], string_data: &str) -> Self {
        let index = attributes
            .iter()
            .find(|a| a.name.local_name == "index")
            .expect("Attributes need to have a index");
        let index = index
            .value
            .parse::<GLuint>()
            .expect("Attribute index is not a number");
        assert!(index <= 16, "Attribute index must be between 0 and 16.");

        let size = attributes
            .iter()
            .find(|a| a.name.local_name == "size")
            .expect("Attributes need to have a size");
        let size = size
            .value
            .parse::<GLint>()
            .expect("Attribute size is not a number");
        assert!(size >= 1, "Attribute size must be between 1 and 5.");
        assert!(size <= 5, "Attribute size must be between 1 and 5.");

        let data_type = attributes
            .iter()
            .find(|a| a.name.local_name == "type")
            .expect("Attributes need to have a type");

        let (data_type, normalized) =
            parse_data_type(&data_type.value).expect("Unknown 'type' field.");

        let integral = attributes.iter().find(|a| a.name.local_name == "integral");
        if let Some(integral) = integral {
            let is_integral = integral
                .name
                .local_name
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
    name: &OwnedAttribute,
    source_attributes: &[OwnedAttribute],
) -> (String, Vec<GLuint>) {
    assert_eq!(name.name.local_name, "name");
    let name = name.value.clone();
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
        let data_type = attributes
            .iter()
            .find(|a| a.name.local_name == "type")
            .expect("Indices need to have a data type");
        let (data_type, _) = parse_data_type(&data_type.value)
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

impl RenderCommand {
    fn render(self, gl: &mut OpenGl) {
        match self {
            RenderCommand::Indexed {
                primitive,
                count,
                index_size,
                offset,
            } => gl.draw_elements(primitive, count, index_size, offset),
            RenderCommand::Normal {
                primitive,
                start,
                end,
            } => gl.draw_arrays(primitive, start, end),
        }
    }
}

struct MeshData {}

pub struct Mesh {
    mesh_data: MeshData,
}

impl Drop for Mesh {
    fn drop(&mut self) {}
}

impl Mesh {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            mesh_data: MeshData {},
        }
    }
    pub fn render(&mut self) {}
    pub fn render_mesh(&mut self, mesh_name: String) {}
    pub fn delete_objects(&mut self) {}
}
