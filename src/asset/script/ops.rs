use indexmap::IndexMap;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::asset::script::{ScriptParamDetails, ScriptParamType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScriptOpcode {
    Known(KnownOpcode),
    Unknown(u32),
}

pub(crate) type ScriptOperationShape = IndexMap<String, ScriptParamDetails>;
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

impl From<ScriptOpcode> for u32 {
    fn from(val: ScriptOpcode) -> Self {
        match val {
            ScriptOpcode::Known(known_opcode) => known_opcode.into(),
            ScriptOpcode::Unknown(val) => val,
        }
    }
}

impl From<u32> for ScriptOpcode {
    fn from(value: u32) -> Self {
        match value.try_into() {
            Ok(known) => ScriptOpcode::Known(known),
            Err(_) => ScriptOpcode::Unknown(value),
        }
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

    CreateKillAllByTagChallenge = 0x1c,

    CreateFindTheGhoulieKeyChallenge = 0x1f,
    // CreateXChallenge = 0x1b,
    SpawnGhoulieWithBox = 0x2a, // Box then Attribs

    CreateWeaponsOnlyChallenge = 0x23,
    CreateFindTheKeyChallenge = 0x27,
    CreateNoBreakHouseChallenge = 0x28,
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
            KnownOpcode::CreateKillAllByTagChallenge => {
                map.insert(
                                    "actor_tag".to_string(),
                                    ScriptParamDetails {
                                        param_type: ScriptParamType::String(0x40),
                                        description: "The tag of the actor which must be killed in the challenge. (eg. objTag_Actor_Zombie)"
                                            .to_string(),
                                    },
                                );

                map.insert(
                                    "unknownU32".to_string(),
                                    ScriptParamDetails {
                                        param_type: ScriptParamType::U32,
                                        description: "Unknown U32 value. Has a value of 1 typically even for kill all challenges."
                                            .to_string(),
                                    },
                                );
            }

            _ => {}
        }

        map
    }

    pub fn operands_size(&self) -> usize {
        match self {
            KnownOpcode::EndScript => 0x00,

            // Challenges
            KnownOpcode::CreateTimeLimitChallenge => 0x4,
            KnownOpcode::CreateKillAllByTagChallenge => 0x40 + 0x4,
            KnownOpcode::CreateFindTheGhoulieKeyChallenge
            | KnownOpcode::CreateFindTheKeyChallenge
            | KnownOpcode::CreateNoBreakHouseChallenge
            | KnownOpcode::CreateWeaponsOnlyChallenge => 0x00,

            KnownOpcode::SetBackground => 0x80,
            KnownOpcode::SetSceneName => 0x48,
            KnownOpcode::SpawnGhoulieWithBox => 0x108,
            KnownOpcode::PlayWalkinCutscene => 0x80,
            KnownOpcode::PlaySound => 0x80,
        }
    }
}
