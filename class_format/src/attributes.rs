//! > # §4.4.7 Attributes
//! Attributes are used in the ClassFile, field_info, method_info, Code_attribute, and
//! record_component_info structures of the class file format (§4.1, §4.5, §4.6, §4.7.3, §4.7.30).
//!
//! Implementation checklist:
//! Critical Attributes:
//! - [ ] ConstantValue
//! - [ ] Code
//! - [ ] StackMapTable
//! - [ ] BootstrapMethods
//! - [ ] NestHost
//! - [ ] NestMembers
//! - [ ] PermittedSubclasses
//!

use crate::read::{BinarySection, Readable};
use crate::simple_grammar;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{self, Error, ErrorKind, Read};

simple_grammar! {

    /// An attribute of indeterminate type. This could be one of the 30 reserved attribute names or a
    /// custom attribute.
    ///
    /// > For all attributes, the attribute_name_index item must be a valid unsigned 16-bit index into
    /// the constant pool of the class. The constant_pool entry at attribute_name_index must be a
    /// CONSTANT_Utf8_info structure (§4.4.7) representing the name of the attribute.
    #[derive(Debug, Clone)]
    pub struct AttributeInfo {
        /// The value of the attribute_name_index item must be a valid index into the constant_pool
        /// table. The constant_pool entry at that index must be a CONSTANT_Utf8_info structure
        /// (§4.4.7).
        name_index: u16,
        info: BinarySection,
    }

    /// The ConstantValue attribute is a fixed-length attribute in the attributes table of a
    /// field_info structure (§4.5). A ConstantValue attribute represents the value of a constant
    /// expression (JLS §15.28), and is used as follows:
    ///  - If the ACC_STATIC flag in the access_flags item of the field_info structure is set, then
    ///    the field represented by the field_info structure is assigned the value represented by
    ///    its ConstantValue attribute as part of the initialization of the class or interface
    ///    declaring the field (§5.5). This occurs prior to the invocation of the class or interface
    ///    initialization method of that class or interface (§2.9.2).
    ///  - Otherwise, the Java Virtual Machine must silently ignore the attribute.
    ///
    /// There may be at most one ConstantValue attribute in the attributes table of a field_info
    /// structure.
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone)]
    pub struct ConstantValue {
        /// The value of the constantvalue_index item must be a valid index into the constant_pool
        /// table. The constant_pool entry at that index gives the value represented by this
        /// attribute. The constant_pool entry must be of a type appropriate to the field, as
        /// specified in Table 4.7.2-A.
        ///
        /// **Table 4.7.2-A. Constant value attribute types**
        /// |            Field Type           |    Entry Type    |
        /// |:-------------------------------:|:----------------:|
        /// | int, short, char, byte, boolean | CONSTANT_Integer |
        /// | float                           | CONSTANT_Float   |
        /// | long                            | CONSTANT_Long    |
        /// | double                          | CONSTANT_Double  |
        /// | String                          | CONSTANT_String  |
        index: u16,
    }


    /// A child struct of the Code attribute that holds the bounds of try catch blocks and the
    /// instruction offset to jump to when an exception is thrown matching the catch type.
    #[derive(Debug, Copy, Clone)]
    pub struct ExceptionBounds {
        /// The values of the two items start_pc and end_pc indicate the ranges in the code array at
        /// which the exception handler is active. The value of start_pc must be a valid index into
        /// the code array of the opcode of an instruction. The value of end_pc either must be a
        /// valid index into the code array of the opcode of an instruction or must be equal to
        /// code_length, the length of the code array. The value of start_pc must be less than the
        /// value of end_pc.
        ///
        /// The start_pc is inclusive and end_pc is exclusive; that is, the exception handler must
        /// be active while the program counter is within the interval [start_pc, end_pc).
        ///
        /// *The fact that end_pc is exclusive is a historical mistake in the design of the Java
        /// Virtual Machine: if the Java Virtual Machine code for a method is exactly 65535 bytes
        /// long and ends with an instruction that is 1 byte long, then that instruction cannot be
        /// protected by an exception handler. A compiler writer can work around this bug by
        /// limiting the maximum size of the generated Java Virtual Machine code for any method,
        /// instance initialization method, or static initializer (the size of any code array) to
        /// 65534 bytes.*
        start_pc: u16,
        /// See documentation for `start_pc`.
        end_pc: u16,
        /// The value of the handler_pc item indicates the start of the exception handler. The value
        /// of the item must be a valid index into the code array and must be the index of the
        /// opcode of an instruction.
        handler_pc: u16,
        /// If the value of the catch_type item is nonzero, it must be a valid index into the
        /// constant_pool table. The constant_pool entry at that index must be a CONSTANT_Class_info
        /// structure (§4.4.1) representing a class of exceptions that this exception handler is
        /// designated to catch. The exception handler will be called only if the thrown exception
        /// is an instance of the given class or one of its subclasses.
        catch_type: u16,
    }

    /// The Code attribute is a variable-length attribute in the attributes table of a method_info
    /// structure (§4.6). A Code attribute contains the Java Virtual Machine instructions and
    /// auxiliary information for a method, including an instance initialization method and a class
    /// or interface initialization method (§2.9.1, §2.9.2).
    ///
    /// If the method is either native or abstract, and is not a class or interface initialization
    /// method, then its method_info structure must not have a Code attribute in its attributes
    /// table. Otherwise, its method_info structure must have exactly one Code attribute in its
    /// attributes table.
    #[doc(strip_hidden)]
    #[derive(Debug, Clone)]
    pub struct Code {
        /// The value of the max_stack item gives the maximum depth of the operand stack of this
        /// method (§2.6.2) at any point during execution of the method.
        max_stack: u16,
        /// The value of the max_locals item gives the number of local variables in the local
        /// variable array allocated upon invocation of this method (§2.6.1), including the local
        /// variables used to pass parameters to the method on its invocation.
        max_locals: u16,
        /// The code array gives the actual bytes of Java Virtual Machine code that implement the
        /// method.
        ///
        /// When the code array is read into memory on a byte-addressable machine, if the first byte
        /// of the array is aligned on a 4-byte boundary, the tableswitch and lookupswitch 32-bit
        /// offsets will be 4-byte aligned. (Refer to the descriptions of those instructions for
        /// more information on the consequences of code array alignment.)
        ///
        /// The detailed constraints on the contents of the code array are extensive and are given
        /// in a separate section (§4.9).
        code: BinarySection,
        /// Each entry in the exception_table array describes one exception handler in the code
        /// array. The order of the handlers in the exception_table array is significant (§2.10).
        exception_table: Vec<ExceptionBounds>,
        /// Each value of the attributes table must be an attribute_info structure (§4.7).
        ///
        /// A Code attribute can have any number of optional attributes associated with it.
        ///
        /// The attributes defined by this specification as appearing in the attributes table of a
        /// Code attribute are listed in Table 4.7-C.
        ///
        /// The rules concerning attributes defined to appear in the attributes table of a Code
        /// attribute are given in §4.7.
        ///
        /// The rules concerning non-predefined attributes in the attributes table of a Code
        /// attribute are given in §4.7.1.
        attributes: Vec<AttributeInfo>,
    }


    /// The StackMapTable attribute is a variable-length attribute in the attributes table of a Code
    /// attribute (§4.7.3). A StackMapTable attribute is used during the process of verification by
    /// type checking (§4.10.1).
    ///
    /// There may be at most one StackMapTable attribute in the attributes table of a Code
    /// attribute.
    ///
    /// In a class file whose version number is 50.0 or above, if a method's Code attribute does not
    /// have a StackMapTable attribute, it has an implicit stack map attribute (§4.10.1). This
    /// implicit stack map attribute is equivalent to a StackMapTable attribute with
    /// number_of_entries equal to zero.
    ///
    /// A stack map frame specifies (either explicitly or implicitly) the bytecode offset at which
    /// it applies, and the verification types of local variables and operand stack entries for that
    /// offset.
    ///
    /// Each stack map frame described in the entries table relies on the previous frame for some of
    /// its semantics. The first stack map frame of a method is implicit, and computed from the
    /// method descriptor by the type checker (§4.10.1.6). The stack_map_frame structure at
    /// entries[0] therefore describes the second stack map frame of the method.
    ///
    /// The bytecode offset at which a stack map frame applies is calculated by taking the value
    /// offset_delta specified in the frame (either explicitly or implicitly), and adding
    /// offset_delta + 1 to the bytecode offset of the previous frame, unless the previous frame is
    /// the initial frame of the method. In that case, the bytecode offset at which the stack map
    /// frame applies is the value offset_delta specified in the frame.
    ///
    /// By using an offset delta rather than storing the actual bytecode offset, we ensure, by
    /// definition, that stack map frames are in the correctly sorted order. Furthermore, by
    /// consistently using the formula offset_delta + 1 for all explicit frames (as opposed to the
    /// implicit first frame), we guarantee the absence of duplicates.
    ///
    /// We say that an instruction in the bytecode has a corresponding stack map frame if the
    /// instruction starts at offset i in the code array of a Code attribute, and the Code attribute
    /// has a StackMapTable attribute whose entries array contains a stack map frame that applies at
    /// bytecode offset i.
    pub struct StackMapTable {
        /// Each entry in the entries table describes one stack map frame of the method. The order
        /// of the stack map frames in the entries table is significant.
        entries: Vec<StackMapFrame>,
    }

    /// The Exceptions attribute is a variable-length attribute in the attributes table of a
    /// method_info structure (§4.6). The Exceptions attribute indicates which checked exceptions a
    /// method may throw.
    ///
    /// There may be at most one Exceptions attribute in the attributes table of a method_info
    /// structure.
    ///
    /// *A method should throw an exception only if at least one of the following three criteria is
    /// met:*
    ///  - *The exception is an instance of RuntimeException or one of its subclasses.*
    ///  - *The exception is an instance of Error or one of its subclasses.*
    ///  - *The exception is an instance of one of the exception classes specified in the
    ///    exception_index_table just described, or one of their subclasses.*
    ///
    /// *These requirements are not enforced in the Java Virtual Machine; they are enforced only at
    /// compile time.*
    pub struct Exceptions {
        /// Each value in the exception_index_table array must be a valid index into the
        /// constant_pool table. The constant_pool entry at that index must be a CONSTANT_Class_info
        /// structure (§4.4.1) representing a class type that this method is declared to throw.
        exception_index_table: Vec<u16>,
    }
}

/// A stack map frame is represented by a discriminated union, stack_map_frame, which consists of a
/// one-byte tag, indicating which item of the union is in use, followed by zero or more bytes,
/// giving more information about the tag.
#[derive(Debug, Clone)]
pub enum StackMapFrame {
    /// The frame type same_frame is represented by tags in the range [0-63]. This frame type
    /// indicates that the frame has exactly the same local variables as the previous frame and that
    /// the operand stack is empty. The offset_delta value for the frame is the value of the tag
    /// item, frame_type.
    SameFrame(u8),
    /// The frame type same_locals_1_stack_item_frame is represented by tags in the range [64, 127].
    /// This frame type indicates that the frame has exactly the same local variables as the
    /// previous frame and that the operand stack has one entry. The offset_delta value for the
    /// frame is given by the formula frame_type - 64. The verification type of the one stack entry
    /// appears after the frame type.
    SameLocals1StackItemFrame {
        frame_type: u8,
        stack: VerificationTypeInfo,
    },
    /// The frame type same_locals_1_stack_item_frame_extended is represented by the tag 247. This
    /// frame type indicates that the frame has exactly the same local variables as the previous
    /// frame and that the operand stack has one entry. The offset_delta value for the frame is
    /// given explicitly, unlike in the frame type same_locals_1_stack_item_frame. The verification
    /// type of the one stack entry appears after offset_delta.
    SameLocals1StackItemFrameExtended {
        offset_delta: u16,
        stack: VerificationTypeInfo,
    },
    /// The frame type chop_frame is represented by tags in the range [248-250]. This frame type
    /// indicates that the frame has the same local variables as the previous frame except that the
    /// last k local variables are absent, and that the operand stack is empty. The value of k is
    /// given by the formula 251 - frame_type. The offset_delta value for the frame is given
    /// explicitly.
    ///
    /// Assume the verification types of local variables in the previous frame are given by locals,
    /// an array structured as in the full_frame frame type. If locals[M-1] in the previous frame
    /// represented local variable X and locals[M] represented local variable Y, then the effect of
    /// removing one local variable is that locals[M-1] in the new frame represents local variable X
    /// and locals[M] is undefined.
    ///
    /// It is an error if k is larger than the number of local variables in locals for the previous
    /// frame, that is, if the number of local variables in the new frame would be less than zero.
    ChopFrame { frame_type: u8, offset_delta: u16 },
    /// The frame type same_frame_extended is represented by the tag 251. This frame type indicates
    /// that the frame has exactly the same local variables as the previous frame and that the
    /// operand stack is empty. The offset_delta value for the frame is given explicitly, unlike in
    /// the frame type same_frame.
    SameFrameExtended { offset_delta: u16 },
    /// The frame type append_frame is represented by tags in the range [252-254]. This frame type
    /// indicates that the frame has the same locals as the previous frame except that k additional
    /// locals are defined, and that the operand stack is empty. The value of k is given by the
    /// formula frame_type - 251. The offset_delta value for the frame is given explicitly.
    ///
    /// The 0th entry in locals represents the verification type of the first additional local
    /// variable. If locals[M] represents local variable N, then:
    ///  - locals[M+1] represents local variable N+1 if locals[M] is one of Top_variable_info,
    ///    Integer_variable_info, Float_variable_info, Null_variable_info,
    ///    UninitializedThis_variable_info, Object_variable_info, or Uninitialized_variable_info;
    ///    and
    ///  - locals[M+1] represents local variable N+2 if locals[M] is either Long_variable_info or
    ///    Double_variable_info.
    ///
    /// **It is an error if, for any index i, locals[i] represents a local variable whose index is
    /// greater than the maximum number of local variables for the method.**
    AppendFrame {
        frame_type: u8,
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
    },
    /// The frame type full_frame is represented by the tag 255. The offset_delta value for the
    /// frame is given explicitly.
    ///
    /// The 0th entry in locals represents the verification type of local variable 0. If locals[M]
    /// represents local variable N, then:
    ///  - locals[M+1] represents local variable N+1 if locals[M] is one of Top_variable_info,
    ///    Integer_variable_info, Float_variable_info, Null_variable_info,
    ///    UninitializedThis_variable_info, Object_variable_info, or Uninitialized_variable_info;
    ///    and
    ///  - locals[M+1] represents local variable N+2 if locals[M] is either Long_variable_info or
    ///    Double_variable_info.
    ///
    /// **It is an error if, for any index i, locals[i] represents a local variable whose index is
    /// greater than the maximum number of local variables for the method.**
    ///
    /// The 0th entry in stack represents the verification type of the bottom of the operand stack,
    /// and subsequent entries in stack represent the verification types of stack entries closer to
    /// the top of the operand stack. We refer to the bottom of the operand stack as stack entry 0,
    /// and to subsequent entries of the operand stack as stack entry 1, 2, etc. If stack[M]
    /// represents stack entry N, then:
    ///  - stack[M+1] represents stack entry N+1 if stack[M] is one of Top_variable_info,
    ///    Integer_variable_info, Float_variable_info, Null_variable_info,
    ///    UninitializedThis_variable_info, Object_variable_info, or Uninitialized_variable_info;
    ///    and
    ///  - stack[M+1] represents stack entry N+2 if stack[M] is either Long_variable_info or
    ///    Double_variable_info.
    ///
    /// **It is an error if, for any index i, stack[i] represents a stack entry whose index is
    /// greater than the maximum operand stack size for the method.**
    FullFrame {
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
        stack: Vec<VerificationTypeInfo>,
    },
}

impl Readable for StackMapFrame {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        let discriminated_type = u8::read(buffer)?;

        Ok(match discriminated_type {
            0..=63 => StackMapFrame::SameFrame(discriminated_type),
            64..=127 => StackMapFrame::SameLocals1StackItemFrame {
                frame_type: discriminated_type,
                stack: VerificationTypeInfo::read(buffer)?,
            },
            128..=246 => {
                return Err(Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "Unknown stack_map_frame tag {} is reserved for future use",
                        discriminated_type
                    ),
                ))
            }
            247 => StackMapFrame::SameLocals1StackItemFrameExtended {
                offset_delta: u16::read(buffer)?,
                stack: VerificationTypeInfo::read(buffer)?,
            },
            248..=250 => StackMapFrame::ChopFrame {
                frame_type: discriminated_type,
                offset_delta: u16::read(buffer)?,
            },
            251 => StackMapFrame::SameFrameExtended {
                offset_delta: u16::read(buffer)?,
            },
            252..=254 => StackMapFrame::AppendFrame {
                frame_type: discriminated_type,
                offset_delta: u16::read(buffer)?,
                locals: {
                    let num_locals = discriminated_type - 251;
                    let mut locals = Vec::with_capacity(num_locals as usize);

                    for _ in 0..num_locals {
                        locals.push(VerificationTypeInfo::read(buffer)?);
                    }

                    locals
                },
            },
            255 => StackMapFrame::FullFrame {
                offset_delta: u16::read(buffer)?,
                locals: <Vec<VerificationTypeInfo>>::read(buffer)?,
                stack: <Vec<VerificationTypeInfo>>::read(buffer)?,
            },
            // The rust compiler knows that this is unreachable but my IDE doesn't so this is here
            // to prevent my IDE from displaying an error on this match.
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        })
    }
}

/// A verification type specifies the type of either one or two locations, where a location is
/// either a single local variable or a single operand stack entry. A verification type is
/// represented by a discriminated union, verification_type_info, that consists of a one-byte tag,
/// indicating which item of the union is in use, followed by zero or more bytes, giving more
/// information about the tag.
#[derive(Debug, Clone, Copy)]
pub enum VerificationTypeInfo {
    Top,
    Integer,
    Float,
    Null,
    UninitializedThis,
    /// The Object_variable_info item indicates that the location has the verification type which is
    /// the class represented by the CONSTANT_Class_info structure (§4.4.1) found in the
    /// constant_pool table at the index given by cpool_index.
    Object {
        const_pool_index: u16,
    },
    /// The Uninitialized_variable_info item indicates that the location has the verification type
    /// uninitialized(Offset). The Offset item indicates the offset, in the code array of the Code
    /// attribute that contains this StackMapTable attribute, of the new instruction (§new) that
    /// created the object being stored in the location.
    Uninitialized {
        offset: u16,
    },
    Long,
    Double,
}

impl Readable for VerificationTypeInfo {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        let discriminated_type = u8::read(buffer)?;

        Ok(match discriminated_type {
            0 => VerificationTypeInfo::Top,
            1 => VerificationTypeInfo::Integer,
            2 => VerificationTypeInfo::Float,
            5 => VerificationTypeInfo::Null,
            6 => VerificationTypeInfo::UninitializedThis,
            7 => VerificationTypeInfo::Object {
                const_pool_index: u16::read(buffer)?,
            },
            8 => VerificationTypeInfo::Uninitialized {
                offset: u16::read(buffer)?,
            },
            4 => VerificationTypeInfo::Long,
            3 => VerificationTypeInfo::Double,
            x => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("{} is not a valid tag for verification_type_info", x),
                ))
            }
        })
    }
}
