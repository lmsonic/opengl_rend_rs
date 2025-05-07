use std::{collections::HashMap, fs::File, io::BufReader, path::Path, str::FromStr};

use gl::types::{GLbyte, GLfloat, GLint, GLshort, GLsizeiptr, GLubyte, GLuint, GLushort};
use glam::bool;
use thiserror::Error;
use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use crate::{
    buffer::{Buffer, Target, Usage},
    opengl::{IndexSize, OpenGl, Primitive},
    vertex_attributes::{DataType, VertexArrayObject, VertexAttribute},
};
type MeshResult<T> = Result<T, MeshError>;

#[derive(Error, Debug)]
pub enum MeshError {
    #[error("Input error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Parsing int data error: {0}")]
    ParseDataError(#[from] std::num::ParseIntError),
    #[error("Parsing float data error: {0}")]
    ParseFloatDataError(#[from] std::num::ParseFloatError),

    #[error("Unimplemented data type {0:?}")]
    UnimplementedDataFormat(DataType),
    #[error("Unknown data type: {0}")]
    UnknownDataType(String),
    #[error("Non existing attribute searched: {0}")]
    NonExistingAttribute(String),
    #[error("Parsing attribute data error: {0}")]
    ParseAttributeDataError(Box<dyn std::error::Error>),
    #[error("Attribute index must be between 0 and 16, found: {0}")]
    InvalidVertexAttributeLocation(GLuint),
    #[error("Attribute size must be between 1 and 5, found: {0}")]
    InvalidVertexAttributeSize(GLint),
    #[error("Parsing bool 'integral' error: {0}")]
    ParseBoolDataError(#[from] std::str::ParseBoolError),
    #[error("Unknown primitive: {0}")]
    UnknownPrimitive(String),
    #[error("`array` 'start' index must be between 0 or greater ,found: {0}")]
    InvalidArrayStart(GLint),
    #[error("`array` 'count' must be between 0 or greater ,found: {0}")]
    InvalidArrayCount(GLint),
    #[error("Bad command element. Must be 'indices' or 'arrays' ,found: {0}")]
    InvalidCommandAttribute(String),
    #[error("Mesh root not found, file path:{0:?}")]
    MeshRootNotFound(String),
    #[error("Mesh has no vertex attributes, file path:{0:?}")]
    NoVertexAttributes(String),
    #[error("Source tags are only valid within vao tags, file path:{0:?}")]
    SourceTagNotInVaoTag(String),
    #[error("This attributes arrays has different size than others, index {0}  file path:{1:?}")]
    VertexAttributesArrayWithDifferentSize(usize, String),
    #[error("could not find source index {0} for vao {1}, file path {2}")]
    VaoSourceInvalidIndex(u32, String, String),
    #[error("cannot be both integral and normalized")]
    IntegralNormalizedError,
    #[error("cannot be both integral and floating point")]
    IntegralFloatingError,
}

#[derive(Debug, PartialEq)]
enum VertexAttributeValues {
    Float(Vec<GLfloat>),
    UnsignedInt(Vec<GLuint>),
    Int(Vec<GLint>),
    UnsignedShort(Vec<GLushort>),
    Short(Vec<GLshort>),
    UnsignedByte(Vec<GLubyte>),
    Byte(Vec<GLbyte>),
}

impl VertexAttributeValues {
    fn parse_add(&mut self, word: &str) -> MeshResult<()> {
        match self {
            Self::Float(items) => {
                items.push(word.parse::<GLfloat>()?);
                Ok(())
            }
            Self::UnsignedInt(items) => {
                items.push(word.parse::<GLuint>()?);
                Ok(())
            }
            Self::Int(items) => {
                items.push(word.parse::<GLint>()?);
                Ok(())
            }
            Self::UnsignedShort(items) => {
                items.push(word.parse::<GLushort>()?);
                Ok(())
            }
            Self::Short(items) => {
                items.push(word.parse::<GLshort>()?);
                Ok(())
            }
            Self::UnsignedByte(items) => {
                items.push(word.parse::<GLubyte>()?);
                Ok(())
            }
            Self::Byte(items) => {
                items.push(word.parse::<GLbyte>()?);
                Ok(())
            }
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::Float(items) => items.len(),
            Self::UnsignedInt(items) => items.len(),
            Self::Int(items) => items.len(),
            Self::UnsignedShort(items) => items.len(),
            Self::Short(items) => items.len(),
            Self::UnsignedByte(items) => items.len(),
            Self::Byte(items) => items.len(),
        }
    }
    fn is_empty(&self) -> bool {
        match self {
            Self::Float(items) => items.is_empty(),
            Self::UnsignedInt(items) => items.is_empty(),
            Self::Int(items) => items.is_empty(),
            Self::UnsignedShort(items) => items.is_empty(),
            Self::Short(items) => items.is_empty(),
            Self::UnsignedByte(items) => items.is_empty(),
            Self::Byte(items) => items.is_empty(),
        }
    }

    fn get_bytes(&self) -> &[u8] {
        match self {
            Self::Float(items) => bytemuck::cast_slice(items),
            Self::UnsignedInt(items) => bytemuck::cast_slice(items),
            Self::Int(items) => bytemuck::cast_slice(items),
            Self::UnsignedShort(items) => bytemuck::cast_slice(items),
            Self::Short(items) => bytemuck::cast_slice(items),
            Self::UnsignedByte(items) => bytemuck::cast_slice(items),
            Self::Byte(items) => bytemuck::cast_slice(items),
        }
    }
}

impl From<DataType> for VertexAttributeValues {
    fn from(value: DataType) -> Self {
        match value {
            DataType::Byte => Self::Byte(vec![]),
            DataType::UnsignedByte => Self::UnsignedByte(vec![]),
            DataType::Short => Self::Short(vec![]),
            DataType::UnsignedShort => Self::UnsignedShort(vec![]),
            DataType::Int => Self::Int(vec![]),
            DataType::UnsignedInt => Self::UnsignedInt(vec![]),
            DataType::Float => Self::Float(vec![]),
            DataType::Fixed | DataType::Double => unimplemented!(),
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, PartialEq)]
enum IndicesValues {
    UnsignedInt(Vec<GLuint>),
    UnsignedShort(Vec<GLushort>),
    UnsignedByte(Vec<GLubyte>),
}

impl IndicesValues {
    fn parse_add(&mut self, word: &str) -> MeshResult<()> {
        match self {
            Self::UnsignedInt(items) => {
                items.push(word.parse::<GLuint>()?);
                Ok(())
            }

            Self::UnsignedShort(items) => {
                items.push(word.parse::<GLushort>()?);
                Ok(())
            }

            Self::UnsignedByte(items) => {
                items.push(word.parse::<GLubyte>()?);
                Ok(())
            }
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::UnsignedInt(items) => items.len(),
            Self::UnsignedShort(items) => items.len(),
            Self::UnsignedByte(items) => items.len(),
        }
    }
    fn get_bytes(&self) -> &[u8] {
        match self {
            Self::UnsignedInt(items) => bytemuck::cast_slice(items),
            Self::UnsignedShort(items) => bytemuck::cast_slice(items),
            Self::UnsignedByte(items) => bytemuck::cast_slice(items),
        }
    }
}

impl From<IndexSize> for IndicesValues {
    fn from(value: IndexSize) -> Self {
        match value {
            IndexSize::UnsignedByte => Self::UnsignedByte(vec![]),
            IndexSize::UnsignedShort => Self::UnsignedShort(vec![]),
            IndexSize::UnsignedInt => Self::UnsignedInt(vec![]),
        }
    }
}

fn parse_attribute_values(data_type: DataType, s: &str) -> MeshResult<VertexAttributeValues> {
    let mut data = VertexAttributeValues::from(data_type);
    for word in s.split_whitespace() {
        data.parse_add(word)?;
    }
    Ok(data)
}
fn parse_indices_values(index_size: IndexSize, s: &str) -> MeshResult<IndicesValues> {
    let mut data = IndicesValues::from(index_size);
    for word in s.split_whitespace() {
        data.parse_add(word)?;
    }
    Ok(data)
}

struct Attribute {
    index: GLuint,
    vertex_attribute: VertexAttribute,
    data: VertexAttributeValues,
}

fn parse_data_type(s: &str) -> MeshResult<(DataType, bool)> {
    match s {
        "float" => Ok((DataType::Float, false)),
        "half" => Ok((DataType::Fixed, false)),
        "int" => Ok((DataType::Int, false)),
        "uint" => Ok((DataType::UnsignedInt, false)),
        "norm-int" => Ok((DataType::Int, true)),
        "norm-uint" => Ok((DataType::UnsignedInt, true)),
        "short" => Ok((DataType::Short, false)),
        "ushort" => Ok((DataType::UnsignedShort, false)),
        "norm-short" => Ok((DataType::Short, true)),
        "norm-ushort" => Ok((DataType::UnsignedShort, true)),
        "byte" => Ok((DataType::Byte, false)),
        "ubyte" => Ok((DataType::UnsignedByte, false)),
        "norm-byte" => Ok((DataType::Byte, true)),
        "norm-ubyte" => Ok((DataType::UnsignedByte, true)),
        _ => Err(MeshError::UnknownDataType(s.to_owned())),
    }
}

fn find_attribute(attributes: &[OwnedAttribute], name: &str) -> MeshResult<String> {
    attributes
        .iter()
        .find(|a| a.name.local_name == name)
        .map(|a| a.value.clone())
        .map_or_else(|| Err(MeshError::NonExistingAttribute(name.to_owned())), Ok)
}

fn find_attribute_parse<T: FromStr>(
    attributes: &[OwnedAttribute],
    name: &str,
) -> Result<T, MeshError>
// this makes the compiler happy
where
    <T as std::str::FromStr>::Err: std::error::Error + 'static,
{
    match find_attribute(attributes, name)?.parse::<T>() {
        Ok(attribute) => Ok(attribute),
        Err(e) => Err(MeshError::ParseAttributeDataError(Box::new(e))),
    }
}
impl Attribute {
    fn new(attributes: &[OwnedAttribute], string_data: &str) -> MeshResult<Self> {
        let index = find_attribute_parse::<GLuint>(attributes, "index")?;
        if index > 16 {
            return Err(MeshError::InvalidVertexAttributeLocation(index));
        }

        let size = find_attribute_parse::<GLint>(attributes, "size")?;
        if !(0..=5).contains(&size) {
            return Err(MeshError::InvalidVertexAttributeSize(size));
        }
        let data_type = find_attribute(attributes, "type")?;
        let (data_type, normalized) = parse_data_type(&data_type)?;

        let integral = find_attribute(attributes, "integral");
        if let Ok(integral) = integral {
            let is_integral = integral.parse::<bool>()?;
            if normalized && is_integral {
                return Err(MeshError::IntegralNormalizedError);
            }
            if data_type.is_floating_point() && is_integral {
                return Err(MeshError::IntegralFloatingError);
            }
        }
        let vertex_attribute = VertexAttribute::new(size, data_type, normalized);
        // parse data
        let data = parse_attribute_values(data_type, string_data)?;
        Ok(Self {
            index,
            vertex_attribute,
            data,
        })
    }

    fn num_elements(&self) -> usize {
        self.data.len() / self.vertex_attribute.components as usize
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
) -> MeshResult<(String, Vec<GLuint>)> {
    let name = find_attribute(vao_attributes, "name")?;
    let mut attributes = vec![];
    for attrib in source_attributes {
        assert_eq!(attrib.name.local_name, "attrib");
        let value = attrib.value.parse::<GLuint>()?;
        attributes.push(value);
    }
    Ok((name, attributes))
}

#[derive(Debug)]
struct IndicesData {
    index_size: IndexSize,
    data: IndicesValues,
}

fn parse_index_type(s: &str) -> MeshResult<(IndexSize, bool)> {
    match s {
        "uint" => Ok((IndexSize::UnsignedInt, false)),
        "norm-uint" => Ok((IndexSize::UnsignedInt, true)),
        "ushort" => Ok((IndexSize::UnsignedShort, false)),
        "norm-ushort" => Ok((IndexSize::UnsignedShort, true)),
        "ubyte" => Ok((IndexSize::UnsignedByte, false)),
        "norm-ubyte" => Ok((IndexSize::UnsignedByte, true)),
        _ => Err(MeshError::UnknownDataType(s.to_owned())),
    }
}

impl IndicesData {
    fn new(attributes: &[OwnedAttribute], string_data: &str) -> MeshResult<Self> {
        let data_type = find_attribute(attributes, "type")?;
        let (index_size, _) = parse_index_type(&data_type)?;

        // parse data
        let data = parse_indices_values(index_size, string_data)?;
        Ok(Self { index_size, data })
    }

    fn byte_size(&self) -> usize {
        self.data.len() * self.index_size.size()
    }
}
#[derive(Debug)]
enum RenderCommand {
    Indexed {
        indexes: IndicesData,
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

fn parse_primitive(s: &str) -> MeshResult<Primitive> {
    match s {
        "lines" => Ok(Primitive::Lines),
        "triangles" => Ok(Primitive::Triangles),
        "tri-strip" => Ok(Primitive::TriangleStrip),
        "tri-fan" => Ok(Primitive::TriangleFan),
        "line-strip" => Ok(Primitive::LineStrip),
        "line-loop" => Ok(Primitive::LineLoop),
        "points" => Ok(Primitive::Points),
        _ => Err(MeshError::UnknownPrimitive(s.to_owned())),
    }
}

impl RenderCommand {
    fn arrays(attributes: &[OwnedAttribute]) -> MeshResult<Self> {
        let primitive = find_attribute(attributes, "cmd")?;
        let primitive = parse_primitive(&primitive)?;

        let start = find_attribute_parse::<GLint>(attributes, "start")?;
        if start < 0 {
            return Err(MeshError::InvalidArrayStart(start));
        }

        let end = find_attribute_parse::<GLint>(attributes, "end")?;
        if end <= 0 {
            return Err(MeshError::InvalidArrayCount(end));
        }
        Ok(Self::Array {
            primitive,
            start,
            end,
        })
    }
    fn indices(attributes: &[OwnedAttribute], indexes: IndicesData) -> MeshResult<Self> {
        let primitive = find_attribute(attributes, "cmd")?;
        let primitive = parse_primitive(&primitive)?;

        let primitive_restart = find_attribute(attributes, "prim-restart")
            .ok()
            .and_then(|s| s.parse::<GLuint>().ok());

        // count, index size, and offset will filled out lated
        Ok(Self::Indexed {
            primitive,
            primitive_restart,
            count: indexes.data.len() as i32,
            index_size: indexes.index_size,
            offset: 0,
            indexes,
        })
    }

    fn render(&mut self, gl: &mut OpenGl) {
        match self {
            Self::Indexed {
                primitive,
                count,
                index_size,
                offset,
                ..
            } => gl.draw_elements(*primitive, *count, *index_size, *offset),
            Self::Array {
                primitive,
                start,
                end,
            } => gl.draw_arrays(*primitive, *start, *end),
        }
    }
}

struct MeshData {
    attrib_array_buffer: Buffer<u8>,
    index_buffer: Buffer<u8>,
    vao: VertexArrayObject,
    named_vaos: HashMap<String, VertexArrayObject>,
    commands: Vec<RenderCommand>,
}

impl MeshData {
    fn new() -> Self {
        Self {
            attrib_array_buffer: Buffer::new(Target::ArrayBuffer),
            index_buffer: Buffer::new(Target::IndexBuffer),
            vao: VertexArrayObject::new(),
            named_vaos: HashMap::new(),
            commands: Vec::new(),
        }
    }
    fn indices(&self) -> Vec<&IndicesData> {
        self.commands
            .iter()
            .filter_map(|cmd| match cmd {
                RenderCommand::Indexed { indexes, .. } => Some(indexes),
                RenderCommand::Array { .. } => None,
            })
            .collect::<Vec<_>>()
    }
}

pub struct Mesh {
    mesh_data: MeshData,
}

struct ParsedData {
    attribs: Vec<Attribute>,
    named_vao_list: Vec<(String, Vec<GLuint>)>,
    commands: Vec<RenderCommand>,
}

impl ParsedData {
    fn indices(&self) -> std::vec::Vec<&IndicesData> {
        self.commands
            .iter()
            .filter_map(|cmd| match cmd {
                RenderCommand::Indexed { indexes, .. } => Some(indexes),
                RenderCommand::Array { .. } => None,
            })
            .collect::<Vec<_>>()
    }
}

impl Mesh {
    #[allow(clippy::too_many_lines)]
    fn parse_xml(path: impl AsRef<Path>) -> MeshResult<ParsedData> {
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

        let mut attribs: Vec<Attribute> = Vec::with_capacity(16);
        // Map from Attribute indices to the indices in the attribs vector just created [0,16]
        let mut named_vao_list: Vec<(String, Vec<GLuint>)> = vec![];
        let mut commands: Vec<RenderCommand> = vec![];

        let path = path.as_ref();
        let string_path = path.as_os_str().to_string_lossy().to_string();
        let file = File::open(path)?;
        let file = BufReader::new(file);

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
                                    return Err(MeshError::MeshRootNotFound(string_path));
                                }
                                parser_state = ParserState::JustPassedMeshRoot;
                            }
                            1 => {
                                let name = name.local_name;
                                if parser_state == ParserState::JustPassedMeshRoot
                                    && name != "attribute"
                                {
                                    return Err(MeshError::NoVertexAttributes(string_path));
                                }

                                parser_state = ParserState::Initial;

                                match name.as_str() {
                                    "attribute" => {
                                        parser_state = ParserState::InAttributeTag { attributes };
                                    }
                                    "vao" => {
                                        parser_state = ParserState::InVaoTag {
                                            vao_attributes: attributes,
                                            sources_attributes: vec![],
                                        };
                                    }

                                    "arrays" => {
                                        let command = RenderCommand::arrays(&attributes)?;
                                        commands.push(command);
                                    }
                                    "indices" => {
                                        parser_state = ParserState::InIndicesTag { attributes };
                                    }
                                    _ => {}
                                }
                            }
                            2 => {
                                if name.local_name == "source" {
                                    match parser_state {
                                        ParserState::InVaoTag {
                                            ref mut sources_attributes,
                                            ..
                                        } => {
                                            sources_attributes.append(&mut attributes);
                                        }
                                        _ => {
                                            return Err(MeshError::SourceTagNotInVaoTag(
                                                string_path,
                                            ))
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                        depth += 1;
                    }
                    XmlEvent::Characters(data) => match parser_state {
                        ParserState::InAttributeTag { attributes } => {
                            let attribute = Attribute::new(&attributes, &data)?;
                            attribs.push(attribute);
                            parser_state = ParserState::Initial;
                        }
                        ParserState::InIndicesTag { attributes } => {
                            let data = IndicesData::new(&attributes, &data)?;
                            let command = RenderCommand::indices(&attributes, data)?;
                            commands.push(command);
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
                                // can only do this at the end
                                let (name, vaos) =
                                    process_vao(&vao_attributes, &sources_attributes)?;
                                named_vao_list.push((name, vaos));
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
        Ok(ParsedData {
            attribs,
            named_vao_list,
            commands,
        })
    }

    pub fn new(path: impl AsRef<Path>) -> MeshResult<Self> {
        let string_path = path.as_ref().as_os_str().to_string_lossy().to_string();

        let parsed_data = Self::parse_xml(path)?;

        let mut mesh_data = MeshData::new();
        mesh_data.commands = parsed_data.commands;

        // checking if vertex attributes have all same sizes
        let mut num_elements = 0;
        for (i, attrib) in parsed_data.attribs.iter().enumerate() {
            if i == 0 {
                num_elements = attrib.num_elements();
            }
            if attrib.num_elements() != num_elements {
                return Err(MeshError::VertexAttributesArrayWithDifferentSize(
                    i,
                    string_path,
                ));
            }
        }

        // this is trying to calculate how much they need to allocate for attributes
        let mut attribute_buffer_size = 0;
        let mut attribute_start_locs = Vec::with_capacity(parsed_data.attribs.len());
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
        }

        mesh_data.vao.bind();
        mesh_data.attrib_array_buffer.bind();
        mesh_data
            .attrib_array_buffer
            .reserve_data_bytes(attribute_buffer_size as GLsizeiptr, Usage::StaticDraw);

        for (i, attrib) in parsed_data.attribs.iter().enumerate() {
            let offset = attribute_start_locs[i];
            mesh_data.attrib_array_buffer.update_data_bytes(
                attrib.data.get_bytes(),
                attrib.byte_size() as isize,
                offset as isize,
            );
            attrib.setup_attribute_array(&mut mesh_data.vao, offset as GLint);
        }

        // fill named vaos
        for (name, source_list) in parsed_data.named_vao_list {
            let mut vao = VertexArrayObject::new();
            vao.bind();
            for attrib in source_list {
                let Some(offset) = parsed_data.attribs.iter().position(|a| a.index == attrib)
                else {
                    return Err(MeshError::VaoSourceInvalidIndex(attrib, name, string_path));
                };

                parsed_data.attribs[offset].setup_attribute_array(&mut vao, offset as GLint);
            }
            mesh_data.named_vaos.insert(name, vao);
        }
        mesh_data.vao.unbind();

        // calculate index buffer size
        let indices_list = mesh_data.commands.iter().filter_map(|cmd| match cmd {
            RenderCommand::Indexed { indexes, .. } => Some(indexes),
            RenderCommand::Array { .. } => None,
        });

        let mut index_buffer_size = 0;
        let mut index_start_locs = vec![];
        for data in indices_list.clone() {
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
                .reserve_data_bytes(index_buffer_size as GLsizeiptr, Usage::StaticDraw);

            // fill in data
            for (i, data) in indices_list.enumerate() {
                let offset = index_start_locs[i];
                mesh_data.index_buffer.update_data_bytes(
                    data.data.get_bytes(),
                    data.byte_size() as isize,
                    offset as isize,
                );
            }
            // fill in indexed rendering commands like said earlier
            // TODO: possibly merge commands and indices in a render pass struct or something like that
            // How to deal with arrays commands thou?
            let mut i = 0;
            for commands in &mut mesh_data.commands {
                if let RenderCommand::Indexed { ref mut offset, .. } = commands {
                    *offset = index_start_locs[i];
                    i += 1;
                }
            }

            for vao in mesh_data.named_vaos.values_mut() {
                vao.bind();
                mesh_data.index_buffer.bind();
            }
            mesh_data.vao.unbind();
        }

        Ok(Self { mesh_data })
    }
    pub fn render(&mut self, gl: &mut OpenGl) {
        self.mesh_data.vao.bind();
        for cmd in &mut self.mesh_data.commands {
            cmd.render(gl);
        }
        self.mesh_data.vao.unbind();
    }
    pub fn render_mesh(&mut self, mesh_name: &str, gl: &mut OpenGl) {
        let Some((_, vao)) = self
            .mesh_data
            .named_vaos
            .iter_mut()
            .find(|(name, _)| **name == mesh_name)
        else {
            return;
        };

        vao.bind();
        for cmd in &mut self.mesh_data.commands {
            cmd.render(gl);
        }
        vao.unbind();
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use gl::types::GLuint;
    use glfw::{fail_on_errors, Context};

    use crate::{
        mesh::RenderCommand,
        opengl::{IndexSize, OpenGl, Primitive},
        vertex_attributes::{DataType, VertexAttribute},
    };

    use super::{Attribute, IndicesData, IndicesValues, Mesh, VertexAttributeValues};
    macro_rules! test_case {
        ($fname:expr) => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/resources/test/", $fname) // assumes Linux ('/')!
        };
    }

    fn test_attribute(
        attribute: &Attribute,
        index: GLuint,
        vertex_attribute: VertexAttribute,
        data: VertexAttributeValues,
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
        match &attribute.data {
            VertexAttributeValues::Float(lhs) => {
                let VertexAttributeValues::Float(rhs) = data else {
                    panic!()
                };
                for (i, attribute) in rhs.iter().enumerate() {
                    let a = attribute;
                    let b = lhs[i];
                    assert!((a - b).abs() < f32::EPSILON, "{i}: {a} {b}");
                }
            }
            _ => assert_eq!(attribute.data, data),
        }
    }

    fn test_indices(indices: &IndicesData, index_size: IndexSize, data: &IndicesValues) {
        assert_eq!(indices.index_size, index_size);
        assert_eq!(indices.data, *data);
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

        let parsed_xml = Mesh::parse_xml(file_path).unwrap();

        // testing attributes
        assert_eq!(parsed_xml.attribs.len(), 1);
        let attribute = &parsed_xml.attribs[0];
        let data = vec![
            0.5, 0.0, -0.5, 0.5, 0.0, 0.5, -0.5, 0.0, 0.5, -0.5, 0.0, -0.5,
        ];

        test_attribute(
            attribute,
            0,
            VertexAttribute::new(3, DataType::Float, false),
            VertexAttributeValues::Float(data),
        );

        // testing indices
        let indices_list: Vec<_> = parsed_xml.indices();
        assert_eq!(indices_list.len(), 1);
        let indices = &indices_list[0];

        let data = IndicesValues::UnsignedShort(vec![0, 1, 2, 0, 2, 1, 2, 3, 0, 2, 0, 3]);

        test_indices(indices, IndexSize::UnsignedShort, &data);

        assert_eq!(parsed_xml.named_vao_list.len(), 0);

        assert_eq!(parsed_xml.commands.len(), 1);
        let cmd = &parsed_xml.commands[0];
        test_commands(cmd, Primitive::Triangles);
    }

    #[test]
    fn test_cube_parse() {
        let file_path = Path::new(test_case!("UnitCube.xml"));

        let parsed_xml = Mesh::parse_xml(file_path).unwrap();

        // testing attributes
        assert_eq!(parsed_xml.attribs.len(), 1);
        let attribute = &parsed_xml.attribs[0];

        // testing attribute data
        let data = VertexAttributeValues::Float(vec![
            0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5,
            0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5,
            0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
        ]);
        test_attribute(
            attribute,
            0,
            VertexAttribute::new(3, DataType::Float, false),
            data,
        );

        // testing indices
        let indices_list: Vec<_> = parsed_xml.indices();
        assert_eq!(indices_list.len(), 1);
        let indices = &indices_list[0];
        assert_eq!(indices.index_size, IndexSize::UnsignedShort);
        let data = IndicesValues::UnsignedShort(vec![
            0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16,
            17, 18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
        ]);

        test_indices(indices, IndexSize::UnsignedShort, &data);

        assert_eq!(parsed_xml.named_vao_list.len(), 0);

        assert_eq!(parsed_xml.commands.len(), 1);
        let cmd = &parsed_xml.commands[0];
        test_commands(cmd, Primitive::Triangles);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_cone_parse() {
        let file_path = Path::new(test_case!("UnitCone.xml"));

        let parsed_xml = Mesh::parse_xml(file_path).unwrap();

        // testing attributes
        assert_eq!(parsed_xml.attribs.len(), 1);
        let attribute = &parsed_xml.attribs[0];

        // testing attribute data
        #[allow(clippy::excessive_precision)]
        #[allow(clippy::unreadable_literal)]
        let data = VertexAttributeValues::Float(vec![
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
        ]);

        test_attribute(
            attribute,
            0,
            VertexAttribute::new(3, DataType::Float, false),
            data,
        );
        // testing indices
        let indices_list: Vec<_> = parsed_xml.indices();

        assert_eq!(indices_list.len(), 2);
        let indices = &indices_list[0];
        assert_eq!(indices.index_size, IndexSize::UnsignedShort);
        let data = IndicesValues::UnsignedShort(vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 1,
        ]);

        test_indices(indices, IndexSize::UnsignedShort, &data);

        let indices = &indices_list[1];
        let data = IndicesValues::UnsignedShort(vec![
            31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10,
            9, 8, 7, 6, 5, 4, 3, 2, 1, 30,
        ]);

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

        let parsed_xml = Mesh::parse_xml(file_path).unwrap();

        // testing attributes
        assert_eq!(parsed_xml.attribs.len(), 2);
        let attribute = &parsed_xml.attribs[0];

        let data = VertexAttributeValues::Float(vec![
            0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5,
            0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5,
            0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
        ]);
        test_attribute(
            attribute,
            0,
            VertexAttribute::new(3, DataType::Float, false),
            data,
        );

        let attribute = &parsed_xml.attribs[1];
        let data = VertexAttributeValues::Float(vec![
            0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0,
            0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0,
            0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0,
            1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0,
            0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0,
            0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0,
        ]);
        test_attribute(
            attribute,
            1,
            VertexAttribute::new(4, DataType::Float, false),
            data,
        );

        // testing indices
        let indices_list: Vec<_> = parsed_xml.indices();

        assert_eq!(indices_list.len(), 1);
        let indices = &indices_list[0];
        let data = IndicesValues::UnsignedShort(vec![
            0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16,
            17, 18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
        ]);
        test_indices(indices, IndexSize::UnsignedShort, &data);

        assert_eq!(parsed_xml.named_vao_list.len(), 0);

        assert_eq!(parsed_xml.commands.len(), 1);
        let cmd = &parsed_xml.commands[0];
        test_commands(cmd, Primitive::Triangles);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_cone_color_parse() {
        let file_path = Path::new(test_case!("UnitConeTint.xml"));

        let parsed_xml = Mesh::parse_xml(file_path).unwrap();

        // testing attributes
        assert_eq!(parsed_xml.attribs.len(), 2);
        let attribute = &parsed_xml.attribs[0];

        // testing attribute data
        #[allow(clippy::excessive_precision)]
        #[allow(clippy::unreadable_literal)]
        let data = VertexAttributeValues::Float(vec![
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
        ]);

        test_attribute(
            attribute,
            0,
            VertexAttribute::new(3, DataType::Float, false),
            data,
        );
        let attribute = &parsed_xml.attribs[1];
        let data = VertexAttributeValues::Float(vec![
            1.0, 1.0, 1.0, 1.0, 0.9, 0.9, 0.9, 1.0, 0.82, 0.82, 0.82, 1.0, 0.74, 0.74, 0.74, 1.0,
            0.66, 0.66, 0.66, 1.0, 0.58, 0.58, 0.58, 1.0, 0.5, 0.5, 0.5, 1.0, 0.58, 0.58, 0.58,
            1.0, 0.66, 0.66, 0.66, 1.0, 0.74, 0.74, 0.74, 1.0, 0.82, 0.82, 0.82, 1.0, 0.9, 0.9,
            0.9, 1.0, 0.82, 0.82, 0.82, 1.0, 0.74, 0.74, 0.74, 1.0, 0.66, 0.66, 0.66, 1.0, 0.58,
            0.58, 0.58, 1.0, 0.5, 0.5, 0.5, 1.0, 0.58, 0.58, 0.58, 1.0, 0.66, 0.66, 0.66, 1.0,
            0.74, 0.74, 0.74, 1.0, 0.82, 0.82, 0.82, 1.0, 0.9, 0.9, 0.9, 1.0, 0.82, 0.82, 0.82,
            1.0, 0.74, 0.74, 0.74, 1.0, 0.66, 0.66, 0.66, 1.0, 0.58, 0.58, 0.58, 1.0, 0.5, 0.5,
            0.5, 1.0, 0.58, 0.58, 0.58, 1.0, 0.66, 0.66, 0.66, 1.0, 0.74, 0.74, 0.74, 1.0, 0.82,
            0.82, 0.82, 1.0, 0.9, 0.9, 0.9, 1.0,
        ]);
        test_attribute(
            attribute,
            1,
            VertexAttribute::new(4, DataType::Float, false),
            data,
        );
        // testing indices
        let indices_list: Vec<_> = parsed_xml.indices();

        assert_eq!(indices_list.len(), 2);
        let indices = &indices_list[0];
        assert_eq!(indices.index_size, IndexSize::UnsignedShort);
        let data = IndicesValues::UnsignedShort(vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 1,
        ]);
        test_indices(indices, IndexSize::UnsignedShort, &data);

        let indices = &indices_list[1];
        let data = IndicesValues::UnsignedShort(vec![
            31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10,
            9, 8, 7, 6, 5, 4, 3, 2, 1, 30,
        ]);
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

        let parsed_xml = Mesh::parse_xml(file_path).unwrap();

        // testing attributes
        assert_eq!(parsed_xml.attribs.len(), 2);
        let attribute = &parsed_xml.attribs[0];

        let data = VertexAttributeValues::Float(vec![
            0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5,
            0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5,
            0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
        ]);
        test_attribute(
            attribute,
            0,
            VertexAttribute::new(3, DataType::Float, false),
            data,
        );

        let attribute = &parsed_xml.attribs[1];
        let data = VertexAttributeValues::Float(vec![
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.75,
            0.75, 0.75, 1.0, 0.75, 0.75, 0.75, 1.0, 0.75, 0.75, 0.75, 1.0, 0.75, 0.75, 0.75, 1.0,
            0.5, 0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 1.0, 1.0,
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.75, 0.75,
            0.75, 1.0, 0.75, 0.75, 0.75, 1.0, 0.75, 0.75, 0.75, 1.0, 0.75, 0.75, 0.75, 1.0, 0.5,
            0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 1.0,
        ]);
        test_attribute(
            attribute,
            1,
            VertexAttribute::new(4, DataType::Float, false),
            data,
        );

        // testing indices
        let indices_list: Vec<_> = parsed_xml.indices();

        assert_eq!(indices_list.len(), 1);
        let indices = &indices_list[0];
        let data = IndicesValues::UnsignedShort(vec![
            0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16,
            17, 18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
        ]);
        test_indices(indices, IndexSize::UnsignedShort, &data);

        assert_eq!(parsed_xml.named_vao_list.len(), 0);

        assert_eq!(parsed_xml.commands.len(), 1);
        let cmd = &parsed_xml.commands[0];
        test_commands(cmd, Primitive::Triangles);
    }
    #[test]
    fn test_sphere_vao() {
        let file_path = Path::new(test_case!("UnitSphere.xml"));

        let parsed_xml = Mesh::parse_xml(file_path).unwrap();
        let expected = [
            ("lit-color", vec![0, 1, 2]),
            ("lit", vec![0, 2]),
            ("color", vec![0, 1]),
            ("flat", vec![0]),
        ];
        test_named_vaos(&parsed_xml.named_vao_list, &expected);
    }

    #[test]
    fn test_buffer_data() {
        let mut glfw = glfw::init(fail_on_errors!()).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));

        // Create a windowed mode window and its OpenGL context
        let (mut window, _) = glfw
            .create_window(600, 600, "OpenGl", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        // Make the window's context current
        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        let _ = OpenGl::new(&mut window);
        let mut mesh = Mesh::new("resources/test/UnitPlane.xml").unwrap();
        mesh.mesh_data.attrib_array_buffer.bind();
        let bytes = mesh.mesh_data.attrib_array_buffer.get_data(0, 48);

        let floats: &[f32] = bytemuck::cast_slice(&bytes);
        assert_eq!(
            floats,
            &[0.5, 0.0, -0.5, 0.5, 0.0, 0.5, -0.5, 0.0, 0.5, -0.5, 0.0, -0.5,]
        );
        mesh.mesh_data.index_buffer.bind();
        let bytes = mesh.mesh_data.index_buffer.get_data(0, 24);
        let indices: &[u16] = bytemuck::cast_slice(&bytes);
        assert_eq!(indices, &[0, 1, 2, 0, 2, 1, 2, 3, 0, 2, 0, 3]);
    }
}
