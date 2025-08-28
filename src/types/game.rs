use num_enum::{IntoPrimitive, TryFromPrimitive};

// Taken from project_grabbed
// https://github.com/x1nixmzeng/project-grabbed
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum AssetType {
    ResTexture = 1,
    ResAnim = 2,
    ResUnknown3 = 3,
    ResModel = 4,
    ResAnimEvents = 5,

    ResCutscene = 7,
    ResCutsceneEvents = 8,

    ResMisc = 10,
    ResActorGoals = 11,
    ResMarker = 12,
    ResFxCallout = 13,
    ResAidList = 14,

    ResLoctext = 16,

    ResXSoundbank = 18,
    ResXDSP = 19,
    ResXCueList = 20,
    ResFont = 21,
    ResGhoulybox = 22,
    ResGhoulyspawn = 23,
    ResScript = 24,
    ResActorAttribs = 25,
    ResEmitter = 26,
    ResParticle = 27,
    ResRumble = 28,
    ResShakeCam = 29,

    ResCount, // This will automatically take the next value (30)
}
