#[derive(Debug, Clone)]
pub enum LinearColour {
    A1R5G5B5 = 0x00000010,
    A4R4G4B4 = 0x0000001D,
    A8 = 0x0000001F,
    A8B8G8R8 = 0x0000003F,
    A8R8G8B8 = 0x00000012,
    B8G8R8A8 = 0x00000040,
    G8B8 = 0x00000017,
    R4G4B4A4 = 0x0000003E,
    R5G5B5A1 = 0x0000003D,
    R5G6B5 = 0x00000011,
    R6G5B5 = 0x00000037,
    R8B8 = 0x00000016,
    R8G8B8A8 = 0x00000041,
    X1R5G5B5 = 0x0000001C,
    X8R8G8B8 = 0x0000001E,
}

#[derive(Debug, Clone)]
enum LinearLuminance {
    A8L8 = 0x00000020,
    AL8 = 0x0000001B,
    L16 = 0x00000035,
    L8 = 0x00000013,

    V16U16 = 0x00000036,
    V8U8 = 0x00000017,
    L6V5U5 = 0x00000037,
    X8L8V8U8 = 0x0000001E,
    Q8W8V8U8 = 0x00000012,

    D24S8 = 0x0000002E,
    F24S8 = 0x0000002F,
    D16 = 0x00000030,
    F16 = 0x00000031,
}

#[derive(Debug, Clone)]
enum Swizzled {
    /* Swizzled formats */
    A8R8G8B8 = 0x00000006,
    X8R8G8B8 = 0x00000007,
    R5G6B5 = 0x00000005,
    R6G5B5 = 0x00000027,
    X1R5G5B5 = 0x00000003,
    A1R5G5B5 = 0x00000002,
    A4R4G4B4 = 0x00000004,
    A8 = 0x00000019,
    A8B8G8R8 = 0x0000003A,
    B8G8R8A8 = 0x0000003B,
    R4G4B4A4 = 0x00000039,
    R5G5B5A1 = 0x00000038,
    R8G8B8A8 = 0x0000003C,
    R8B8 = 0x00000029,
    G8B8 = 0x00000028,
}

#[derive(Debug, Clone)]
pub enum StandardFormat {
    Unknown = 0xFFFFFFFF,

    P8 = 0x0000000B,

    L8 = 0x00000000,
    A8L8 = 0x0000001A,
    AL8 = 0x00000001,
    L16 = 0x00000032,

    V8U8 = 0x00000028,
    L6V5U5 = 0x00000027,
    X8L8V8U8 = 0x00000007,
    Q8W8V8U8 = 0x0000003A,
    V16U16 = 0x00000033,

    D16 = 0x0000002C,
    D24S8 = 0x0000002A,
    F16 = 0x0000002D,
    F24S8 = 0x0000002B,

    /* YUV formats */
    YUY2 = 0x00000024,
    UYVY = 0x00000025,

    /* Compressed formats */
    DXT1 = 0x0000000C,
    DXT2Or3 = 0x0000000E,
    DXT4Or5 = 0x0000000F,
}

#[derive(Debug, Clone)]
#[repr(u32)]
pub enum D3DFormat {
    Swizzled(Swizzled),
    Luminance(LinearLuminance),
    Standard(StandardFormat),
    Linear(LinearColour),

    VertexData = 100,
    Index16 = 101,
    ForceDWORD = 0x7fffffff,
}
