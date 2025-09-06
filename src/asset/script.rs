use std::io::{Cursor, Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use indexmap::IndexMap;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    VirtualResource,
    asset::{Asset, AssetDescriptor, AssetParseError},
    game::AssetType,
};

#[derive(Debug, Clone)]
pub struct ScriptDescriptor {
    operations: Vec<ScriptOperation>,
}

impl ScriptDescriptor {
    pub fn operations(&self) -> &[ScriptOperation] {
        &self.operations
    }

    pub fn operations_mut(&mut self) -> &mut Vec<ScriptOperation> {
        &mut self.operations
    }
}

#[derive(Debug, Clone)]
pub enum ScriptError {
    SizeMismatch,
    InvalidInput,
    UnsupportedOutputType,
}

#[derive(Debug)]
pub struct Script {
    name: String,
    descriptor: ScriptDescriptor,
    data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScriptOpcode {
    Known(KnownOpcode),
    Unknown(u32),
}

impl Into<u32> for ScriptOpcode {
    fn into(self) -> u32 {
        match self {
            ScriptOpcode::Known(known_opcode) => known_opcode.into(),
            ScriptOpcode::Unknown(val) => val,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScriptOperation {
    size: u32,
    opcode: ScriptOpcode,
    operand_bytes: Vec<u8>,
}

impl ScriptOperation {
    pub fn get_shape(&self) -> ScriptOperationShape {
        self.opcode.get_shape()
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn opcode(&self) -> &ScriptOpcode {
        &self.opcode
    }

    pub fn operand_bytes(&self) -> &[u8] {
        &self.operand_bytes
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let size = self.operand_bytes.len() + 8;

        let mut bytes = vec![0x00; size];

        let mut cur = Cursor::new(&mut bytes[..]);

        cur.write_u32::<LittleEndian>(size as u32);
        cur.write_u32::<LittleEndian>(self.opcode.into());
        cur.write_all(self.operand_bytes());

        bytes
    }

    pub fn operand_bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.operand_bytes
    }
}

#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive, PartialEq)]
#[repr(u32)]
pub enum KnownOpcode {
    EndScript = 0x0,
    SetBackground = 0x1,

    SetSceneName = 0xa,

    // SetPlayState = 0xe, // eg. Free Play
    // Signal0f = 0x0f,
    // Signal11 = 0x11,
    // Signal18 = 0x18,
    CreateTimeLimitChallenge = 0x1a,
    // CreateXChallenge = 0x1b,
    SpawnGhoulieWithBox = 0x2a, // Box then Attribs

    // Signal2f = 0x2f,
    // Signal30 = 0x30,

    // g10x32 = 0x32,
    // g10x33 = 0x33,
    // g10x34 = 0x34,
    // g10x35 = 0x35,
    // g10x36 = 0x36,
    // g10x37 = 0x37,
    // g10x38 = 0x38,

    // Unknown39 = 0x39,
    // Signal3b = 0x3b,
    // Signal3c = 0x3c,

    // Signal45 = 0x45,
    PlayWalkinCutscene = 0x53, // ?

    // SetChallengeId = 0x7a,
    PlaySound = 0x8d,
}

impl From<u32> for ScriptOpcode {
    fn from(value: u32) -> Self {
        match value.try_into() {
            Ok(known) => ScriptOpcode::Known(known),
            Err(_) => ScriptOpcode::Unknown(value),
        }
    }
}

impl AssetDescriptor for ScriptDescriptor {
    fn from_bytes(data: &[u8]) -> Result<Self, AssetParseError> {
        if data.len() < 8 {
            return Err(AssetParseError::InputTooSmall);
        }

        let mut cur = Cursor::new(data);

        let mut operations = Vec::new();

        let mut size = cur.read_u32::<LittleEndian>()?;
        let mut opcode = cur.read_u32::<LittleEndian>()?;

        while opcode != 0 {
            if size < 8 {
                return Err(AssetParseError::ErrorParsingDescriptor);
            }

            let mut operand_bytes = vec![0x00; (size as usize) - 8];
            cur.read_exact(&mut operand_bytes)?;

            operations.push(ScriptOperation {
                size,
                opcode: opcode.into(),
                operand_bytes,
            });

            size = cur.read_u32::<LittleEndian>()?;
            opcode = cur.read_u32::<LittleEndian>()?;
        }

        if size == 8 && opcode == 0 {
            operations.push(ScriptOperation {
                size: 8,
                opcode: ScriptOpcode::Known(KnownOpcode::EndScript),
                operand_bytes: [].to_vec(),
            });
        } else {
            // Size mismatch
            return Err(AssetParseError::ErrorParsingDescriptor);
        }

        // TODO: Sanity check the read length here
        Ok(ScriptDescriptor { operations })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, AssetParseError> {
        let mut bytes = Vec::new();

        self.operations
            .iter()
            .map(|op| op.to_bytes())
            .for_each(|b| bytes.extend_from_slice(&b));

        Ok(bytes)
    }

    fn size(&self) -> usize {
        self.operations().iter().map(|v| v.size() as usize).sum()
    }

    fn asset_type() -> AssetType {
        AssetType::ResScript
    }
}

#[derive(Debug)]
pub enum ScriptParamType {
    F32,
    F64,
    U8,
    I8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,

    String(usize),
    WString(usize),
    Bytes(usize),
}

#[derive(Debug)]
pub struct ScriptParamDetails {
    param_type: ScriptParamType,
    description: String,
}

impl ScriptParamDetails {
    pub fn param_type(&self) -> &ScriptParamType {
        &self.param_type
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

type ScriptOperationShape = IndexMap<String, ScriptParamDetails>;

impl ScriptOpcode {
    pub fn get_shape(&self) -> ScriptOperationShape {
        match self {
            ScriptOpcode::Known(known_opcode) => known_opcode.get_shape(),
            ScriptOpcode::Unknown(_) => {
                IndexMap::new() // Return an empty hashmap to indicate no shape
            }
        }
    }
}

impl Asset for Script {
    type Descriptor = ScriptDescriptor;

    fn new(
        name: &str,
        descriptor: &Self::Descriptor,
        virtual_res: &VirtualResource,
    ) -> Result<Self, AssetParseError> {
        Ok(Script {
            name: name.to_string(),
            descriptor: descriptor.clone(),
            data: virtual_res.get_all_bytes(),
        })
    }

    fn descriptor(&self) -> &Self::Descriptor {
        &self.descriptor
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn resource_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

impl KnownOpcode {
    pub fn get_shape(&self) -> ScriptOperationShape {
        let mut map: ScriptOperationShape = IndexMap::new();

        match self {
            KnownOpcode::EndScript => {}
            KnownOpcode::SetBackground => {
                map.insert("background_aid".to_string(), ScriptParamDetails {
                            param_type: ScriptParamType::String(0x80),
                            description:
                                "The asset ID of the background to be loaded at the beginning of the scene."
                                    .to_string(),
                        });
            }
            KnownOpcode::SetSceneName => {
                map.insert(
                    "scene_name".to_string(),
                    ScriptParamDetails {
                        param_type: ScriptParamType::String(0x40),
                        description:
                            "The name of the current scene as a string (eg. Scummy Scullery)"
                                .to_string(),
                    },
                );
                map.insert(
                    "unknown1".to_string(),
                    ScriptParamDetails {
                        param_type: ScriptParamType::Bytes(4),
                        description: "Unknown value of size 4 bytes. Suspected to be a u32."
                            .to_string(),
                    },
                );
                map.insert(
                    "unknown2".to_string(),
                    ScriptParamDetails {
                        param_type: ScriptParamType::Bytes(4),
                        description: "Unknown value of size 4 bytes. Suspected to be a f32."
                            .to_string(),
                    },
                );
                map.insert(
                    "unknown3".to_string(),
                    ScriptParamDetails {
                        param_type: ScriptParamType::Bytes(4),
                        description: "Unknown value of size 4 bytes. Suspected to be a f32."
                            .to_string(),
                    },
                );
            }
            KnownOpcode::CreateTimeLimitChallenge => {
                map.insert(
                    "duration".to_string(),
                    ScriptParamDetails {
                        param_type: ScriptParamType::F32,
                        description: "The duration of the timer in the challenge.".to_string(),
                    },
                );
            }
            KnownOpcode::SpawnGhoulieWithBox => {
                map.insert(
                    "ghoulybox_aid".to_string(),
                    ScriptParamDetails {
                        param_type: ScriptParamType::String(0x80),
                        description: "The asset ID of the ghoulybox that will be spawned."
                            .to_string(),
                    },
                );

                map.insert(
                    "spawn_count".to_string(),
                    ScriptParamDetails {
                        param_type: ScriptParamType::U32,
                        description: "The number of entities spawned? (Not 100% sure on this)"
                            .to_string(),
                    },
                );

                map.insert(
                    "actor_attribs_aid".to_string(),
                    ScriptParamDetails {
                        param_type: ScriptParamType::String(0x80),
                        description: "The asset ID of the actor attribs asset that will be used."
                            .to_string(),
                    },
                );
            }
            KnownOpcode::PlayWalkinCutscene => {
                map.insert(
                            "cutscene_aid".to_string(),
                            ScriptParamDetails {
                                param_type: ScriptParamType::String(0x80),
                                description: "The asset ID of the cutscene to be played on room walk in (eg. aid_cutscene_ghoulies_roomwalkins_walkina)".to_string(),
                            },
                        );
            }
            KnownOpcode::PlaySound => {
                map.insert(
                            "soundbank_id".to_string(),
                            ScriptParamDetails {
                                param_type: ScriptParamType::String(0x80),
                                description: "The soundbank ID of the audio to be played. (eg. XACT_SOUNDBANK_GZOMBIE_DISAPPOINTED)"
                                    .to_string(),
                            },
                        );
            }
        }

        map
    }

    pub fn operands_size(&self) -> usize {
        match self {
            KnownOpcode::EndScript => 0x00,
            KnownOpcode::SetBackground => 0x80,
            KnownOpcode::SetSceneName => 0x48,
            KnownOpcode::CreateTimeLimitChallenge => 0x4,
            KnownOpcode::SpawnGhoulieWithBox => 0x108,
            KnownOpcode::PlayWalkinCutscene => 0x80,
            KnownOpcode::PlaySound => 0x80,
        }
    }
}
