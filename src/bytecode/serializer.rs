use super::{Chunk, OpCode, Value, Function};
use std::io::{Write, Read, Result as IoResult, Error, ErrorKind};

/// Zero字节码文件魔数 "ZERO"
const MAGIC: [u8; 4] = [0x5A, 0x45, 0x52, 0x4F];
const VERSION_MAJOR: u16 = 0;
const VERSION_MINOR: u16 = 1;

/// 字节码序列化器
pub struct BytecodeSerializer;

impl BytecodeSerializer {
    /// 将Chunk序列化为字节码文件
    pub fn serialize<W: Write>(chunk: &Chunk, writer: &mut W) -> IoResult<()> {
        // 写入文件头
        writer.write_all(&MAGIC)?;
        writer.write_all(&VERSION_MAJOR.to_le_bytes())?;
        writer.write_all(&VERSION_MINOR.to_le_bytes())?;
        writer.write_all(&(chunk.constants.len() as u32).to_le_bytes())?;
        writer.write_all(&(chunk.code.len() as u32).to_le_bytes())?;

        // 写入常量池
        for constant in &chunk.constants {
            Self::write_value(constant, writer)?;
        }

        // 写入指令序列
        for opcode in &chunk.code {
            Self::write_opcode(opcode, writer)?;
        }

        // 写入行号信息
        for line in &chunk.lines {
            writer.write_all(&(*line as u32).to_le_bytes())?;
        }

        Ok(())
    }

    /// 写入Value
    fn write_value<W: Write>(value: &Value, writer: &mut W) -> IoResult<()> {
        match value {
            Value::Integer(i) => {
                writer.write_all(&[0x01])?; // Type ID
                writer.write_all(&i.to_le_bytes())?;
            }
            Value::Float(f) => {
                writer.write_all(&[0x02])?;
                writer.write_all(&f.to_le_bytes())?;
            }
            Value::String(s) => {
                writer.write_all(&[0x03])?;
                let bytes = s.as_bytes();
                writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
                writer.write_all(bytes)?;
            }
            Value::Boolean(b) => {
                writer.write_all(&[0x04])?;
                writer.write_all(&[if *b { 1 } else { 0 }])?;
            }
            Value::Char(c) => {
                writer.write_all(&[0x09])?;
                let mut buf = [0u8; 4];
                let encoded = c.encode_utf8(&mut buf);
                writer.write_all(&(encoded.len() as u8).to_le_bytes())?;
                writer.write_all(encoded.as_bytes())?;
            }
            Value::Array(arr) => {
                writer.write_all(&[0x05])?;
                writer.write_all(&(arr.len() as u32).to_le_bytes())?;
                for elem in arr {
                    Self::write_value(elem, writer)?;
                }
            }
            Value::Function(func) => {
                writer.write_all(&[0x06])?;
                Self::write_function(func, writer)?;
            }
            Value::Struct(s) => {
                writer.write_all(&[0x08])?;
                let name_bytes = s.struct_name.as_bytes();
                writer.write_all(&(name_bytes.len() as u32).to_le_bytes())?;
                writer.write_all(name_bytes)?;
                writer.write_all(&(s.fields.len() as u32).to_le_bytes())?;
                for field in &s.fields {
                    Self::write_value(field, writer)?;
                }
            }
            Value::Null => {
                writer.write_all(&[0x07])?;
            }
        }
        Ok(())
    }

    /// 写入Function
    fn write_function<W: Write>(func: &Function, writer: &mut W) -> IoResult<()> {
        // 写入函数名
        let name_bytes = func.name.as_bytes();
        writer.write_all(&(name_bytes.len() as u32).to_le_bytes())?;
        writer.write_all(name_bytes)?;

        // 写入参数数量和局部变量数量
        writer.write_all(&(func.arity as u32).to_le_bytes())?;
        writer.write_all(&(func.locals_count as u32).to_le_bytes())?;

        // 递归写入函数的Chunk
        writer.write_all(&(func.chunk.constants.len() as u32).to_le_bytes())?;
        writer.write_all(&(func.chunk.code.len() as u32).to_le_bytes())?;

        for constant in &func.chunk.constants {
            Self::write_value(constant, writer)?;
        }

        for opcode in &func.chunk.code {
            Self::write_opcode(opcode, writer)?;
        }

        for line in &func.chunk.lines {
            writer.write_all(&(*line as u32).to_le_bytes())?;
        }

        Ok(())
    }

    /// 写入OpCode
    fn write_opcode<W: Write>(opcode: &OpCode, writer: &mut W) -> IoResult<()> {
        match opcode {
            OpCode::LoadConst(idx) => {
                writer.write_all(&[0x00])?;
                writer.write_all(&(*idx as u32).to_le_bytes())?;
            }
            OpCode::LoadNull => writer.write_all(&[0x01])?,
            OpCode::LoadLocal(slot) => {
                writer.write_all(&[0x02])?;
                writer.write_all(&(*slot as u32).to_le_bytes())?;
            }
            OpCode::StoreLocal(slot) => {
                writer.write_all(&[0x03])?;
                writer.write_all(&(*slot as u32).to_le_bytes())?;
            }
            OpCode::LoadGlobal(idx) => {
                writer.write_all(&[0x04])?;
                writer.write_all(&(*idx as u32).to_le_bytes())?;
            }
            OpCode::StoreGlobal(idx) => {
                writer.write_all(&[0x05])?;
                writer.write_all(&(*idx as u32).to_le_bytes())?;
            }
            OpCode::Add => writer.write_all(&[0x10])?,
            OpCode::Subtract => writer.write_all(&[0x11])?,
            OpCode::Multiply => writer.write_all(&[0x12])?,
            OpCode::Divide => writer.write_all(&[0x13])?,
            OpCode::Modulo => writer.write_all(&[0x14])?,
            OpCode::Negate => writer.write_all(&[0x15])?,
            OpCode::Equal => writer.write_all(&[0x20])?,
            OpCode::NotEqual => writer.write_all(&[0x21])?,
            OpCode::Greater => writer.write_all(&[0x22])?,
            OpCode::GreaterEqual => writer.write_all(&[0x23])?,
            OpCode::Less => writer.write_all(&[0x24])?,
            OpCode::LessEqual => writer.write_all(&[0x25])?,
            OpCode::Not => writer.write_all(&[0x30])?,
            OpCode::And => writer.write_all(&[0x31])?,
            OpCode::Or => writer.write_all(&[0x32])?,
            OpCode::Jump(offset) => {
                writer.write_all(&[0x40])?;
                writer.write_all(&(*offset as u32).to_le_bytes())?;
            }
            OpCode::JumpIfFalse(offset) => {
                writer.write_all(&[0x41])?;
                writer.write_all(&(*offset as u32).to_le_bytes())?;
            }
            OpCode::JumpIfTrue(offset) => {
                writer.write_all(&[0x42])?;
                writer.write_all(&(*offset as u32).to_le_bytes())?;
            }
            OpCode::Loop(offset) => {
                writer.write_all(&[0x43])?;
                writer.write_all(&(*offset as u32).to_le_bytes())?;
            }
            OpCode::Call(argc) => {
                writer.write_all(&[0x50])?;
                writer.write_all(&(*argc as u32).to_le_bytes())?;
            }
            OpCode::Return => writer.write_all(&[0x51])?,
            OpCode::NewArray(size) => {
                writer.write_all(&[0x60])?;
                writer.write_all(&(*size as u32).to_le_bytes())?;
            }
            OpCode::ArrayGet => writer.write_all(&[0x61])?,
            OpCode::ArraySet => writer.write_all(&[0x62])?,
            OpCode::ArrayLen => writer.write_all(&[0x63])?,
            OpCode::NewStruct(field_count) => {
                writer.write_all(&[0x64])?;
                writer.write_all(&(*field_count as u32).to_le_bytes())?;
            }
            OpCode::FieldGet(idx) => {
                writer.write_all(&[0x65])?;
                writer.write_all(&(*idx as u32).to_le_bytes())?;
            }
            OpCode::FieldSet(idx) => {
                writer.write_all(&[0x66])?;
                writer.write_all(&(*idx as u32).to_le_bytes())?;
            }
            OpCode::Pop => writer.write_all(&[0x70])?,
            OpCode::Dup => writer.write_all(&[0x71])?,
            OpCode::Print => writer.write_all(&[0xF0])?,
            OpCode::Halt => writer.write_all(&[0xFF])?,
        }
        Ok(())
    }
}

/// 字节码反序列化器
pub struct BytecodeDeserializer;

impl BytecodeDeserializer {
    /// 从字节码文件反序列化为Chunk
    pub fn deserialize<R: Read>(reader: &mut R) -> IoResult<Chunk> {
        // 读取并验证文件头
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if magic != MAGIC {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid magic number"));
        }

        let mut version_major = [0u8; 2];
        let mut version_minor = [0u8; 2];
        reader.read_exact(&mut version_major)?;
        reader.read_exact(&mut version_minor)?;

        let ver_major = u16::from_le_bytes(version_major);
        let ver_minor = u16::from_le_bytes(version_minor);

        if ver_major != VERSION_MAJOR {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unsupported version {}.{}", ver_major, ver_minor),
            ));
        }

        // 读取常量和指令数量
        let constants_count = Self::read_u32(reader)?;
        let code_count = Self::read_u32(reader)?;

        // 读取常量池
        let mut constants = Vec::with_capacity(constants_count as usize);
        for _ in 0..constants_count {
            constants.push(Self::read_value(reader)?);
        }

        // 读取指令序列
        let mut code = Vec::with_capacity(code_count as usize);
        for _ in 0..code_count {
            code.push(Self::read_opcode(reader)?);
        }

        // 读取行号信息
        let mut lines = Vec::with_capacity(code_count as usize);
        for _ in 0..code_count {
            lines.push(Self::read_u32(reader)? as usize);
        }

        Ok(Chunk {
            code,
            constants,
            lines,
        })
    }

    /// 读取Value
    fn read_value<R: Read>(reader: &mut R) -> IoResult<Value> {
        let mut type_id = [0u8; 1];
        reader.read_exact(&mut type_id)?;

        match type_id[0] {
            0x01 => {
                let mut bytes = [0u8; 8];
                reader.read_exact(&mut bytes)?;
                Ok(Value::Integer(i64::from_le_bytes(bytes)))
            }
            0x02 => {
                let mut bytes = [0u8; 8];
                reader.read_exact(&mut bytes)?;
                Ok(Value::Float(f64::from_le_bytes(bytes)))
            }
            0x03 => {
                let len = Self::read_u32(reader)? as usize;
                let mut bytes = vec![0u8; len];
                reader.read_exact(&mut bytes)?;
                String::from_utf8(bytes)
                    .map(Value::String)
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))
            }
            0x04 => {
                let mut byte = [0u8; 1];
                reader.read_exact(&mut byte)?;
                Ok(Value::Boolean(byte[0] != 0))
            }
            0x09 => {
                let mut len_byte = [0u8; 1];
                reader.read_exact(&mut len_byte)?;
                let len = len_byte[0] as usize;
                let mut bytes = vec![0u8; len];
                reader.read_exact(&mut bytes)?;
                let s = String::from_utf8(bytes)
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
                Ok(Value::Char(s.chars().next().unwrap_or('\0')))
            }
            0x05 => {
                let len = Self::read_u32(reader)? as usize;
                let mut arr = Vec::with_capacity(len);
                for _ in 0..len {
                    arr.push(Self::read_value(reader)?);
                }
                Ok(Value::Array(arr))
            }
            0x06 => Ok(Value::Function(Self::read_function(reader)?)),
            0x07 => Ok(Value::Null),
            0x08 => {
                let name_len = Self::read_u32(reader)? as usize;
                let mut name_bytes = vec![0u8; name_len];
                reader.read_exact(&mut name_bytes)?;
                let struct_name = String::from_utf8(name_bytes)
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
                
                let field_count = Self::read_u32(reader)? as usize;
                let mut fields = Vec::with_capacity(field_count);
                for _ in 0..field_count {
                    fields.push(Self::read_value(reader)?);
                }
                Ok(Value::Struct(crate::bytecode::StructValue {
                    struct_name,
                    fields,
                }))
            }
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unknown value type: 0x{:02X}", type_id[0]),
            )),
        }
    }

    /// 读取Function
    fn read_function<R: Read>(reader: &mut R) -> IoResult<Function> {
        // 读取函数名
        let name_len = Self::read_u32(reader)? as usize;
        let mut name_bytes = vec![0u8; name_len];
        reader.read_exact(&mut name_bytes)?;
        let name = String::from_utf8(name_bytes)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        // 读取参数和局部变量数量
        let arity = Self::read_u32(reader)? as usize;
        let locals_count = Self::read_u32(reader)? as usize;

        // 读取函数的Chunk
        let constants_count = Self::read_u32(reader)?;
        let code_count = Self::read_u32(reader)?;

        let mut constants = Vec::with_capacity(constants_count as usize);
        for _ in 0..constants_count {
            constants.push(Self::read_value(reader)?);
        }

        let mut code = Vec::with_capacity(code_count as usize);
        for _ in 0..code_count {
            code.push(Self::read_opcode(reader)?);
        }

        let mut lines = Vec::with_capacity(code_count as usize);
        for _ in 0..code_count {
            lines.push(Self::read_u32(reader)? as usize);
        }

        Ok(Function {
            name,
            arity,
            chunk: Chunk {
                code,
                constants,
                lines,
            },
            locals_count,
        })
    }

    /// 读取OpCode
    fn read_opcode<R: Read>(reader: &mut R) -> IoResult<OpCode> {
        let mut opcode = [0u8; 1];
        reader.read_exact(&mut opcode)?;

        match opcode[0] {
            0x00 => Ok(OpCode::LoadConst(Self::read_u32(reader)? as usize)),
            0x01 => Ok(OpCode::LoadNull),
            0x02 => Ok(OpCode::LoadLocal(Self::read_u32(reader)? as usize)),
            0x03 => Ok(OpCode::StoreLocal(Self::read_u32(reader)? as usize)),
            0x04 => Ok(OpCode::LoadGlobal(Self::read_u32(reader)? as usize)),
            0x05 => Ok(OpCode::StoreGlobal(Self::read_u32(reader)? as usize)),
            0x10 => Ok(OpCode::Add),
            0x11 => Ok(OpCode::Subtract),
            0x12 => Ok(OpCode::Multiply),
            0x13 => Ok(OpCode::Divide),
            0x14 => Ok(OpCode::Modulo),
            0x15 => Ok(OpCode::Negate),
            0x20 => Ok(OpCode::Equal),
            0x21 => Ok(OpCode::NotEqual),
            0x22 => Ok(OpCode::Greater),
            0x23 => Ok(OpCode::GreaterEqual),
            0x24 => Ok(OpCode::Less),
            0x25 => Ok(OpCode::LessEqual),
            0x30 => Ok(OpCode::Not),
            0x31 => Ok(OpCode::And),
            0x32 => Ok(OpCode::Or),
            0x40 => Ok(OpCode::Jump(Self::read_u32(reader)? as usize)),
            0x41 => Ok(OpCode::JumpIfFalse(Self::read_u32(reader)? as usize)),
            0x42 => Ok(OpCode::JumpIfTrue(Self::read_u32(reader)? as usize)),
            0x43 => Ok(OpCode::Loop(Self::read_u32(reader)? as usize)),
            0x50 => Ok(OpCode::Call(Self::read_u32(reader)? as usize)),
            0x51 => Ok(OpCode::Return),
            0x60 => Ok(OpCode::NewArray(Self::read_u32(reader)? as usize)),
            0x61 => Ok(OpCode::ArrayGet),
            0x62 => Ok(OpCode::ArraySet),
            0x63 => Ok(OpCode::ArrayLen),
            0x64 => Ok(OpCode::NewStruct(Self::read_u32(reader)? as usize)),
            0x65 => Ok(OpCode::FieldGet(Self::read_u32(reader)? as usize)),
            0x66 => Ok(OpCode::FieldSet(Self::read_u32(reader)? as usize)),
            0x70 => Ok(OpCode::Pop),
            0x71 => Ok(OpCode::Dup),
            0xF0 => Ok(OpCode::Print),
            0xFF => Ok(OpCode::Halt),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unknown opcode: 0x{:02X}", opcode[0]),
            )),
        }
    }

    /// 辅助方法：读取u32
    fn read_u32<R: Read>(reader: &mut R) -> IoResult<u32> {
        let mut bytes = [0u8; 4];
        reader.read_exact(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }
}