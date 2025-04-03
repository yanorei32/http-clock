use binrw::{binrw, BinRead, BinWrite};
use bitfield_struct::bitfield;

#[bitfield(u8)]
pub struct HeaderPacked {
    #[bits(3)]
    pub size_of_global_color_table: u32,

    #[bits(1)]
    pub sort_flag: bool,

    /// WARNING: Most GIF parser supports 8 bpp only.
    #[bits(3)]
    pub color_resolution: usize,

    #[bits(1)]
    pub global_color_table_flag: bool,
}

impl BinRead for HeaderPacked {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let mut packed = [0u8; 1];
        reader.read_exact(&mut packed)?;
        Ok(HeaderPacked::from_bits(packed[0]))
    }
}

pub struct HeaderPackedWriteArgs {
    global_color_table_length: usize,
}

impl BinWrite for HeaderPacked {
    type Args<'a> = HeaderPackedWriteArgs;

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        _endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        writer.write_all(&[self
            .with_global_color_table_flag(args.global_color_table_length != 0)
            .with_size_of_global_color_table(if args.global_color_table_length == 0 {
                0
            } else {
                args.global_color_table_length.ilog2() - 1
            })
            .into_bits()])?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Version {
    GIF87a,
    GIF89a,
}

impl BinRead for Version {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let mut version = [0u8; 3];

        reader.read_exact(&mut version)?;

        Ok(match &version {
            b"87a" => Self::GIF87a,
            b"89a" => Self::GIF89a,
            version => panic!("{version:?} is not supported."),
        })
    }
}

impl BinWrite for Version {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        match self {
            Self::GIF87a => writer.write_all(b"87a")?,
            Self::GIF89a => writer.write_all(b"89a")?,
        };

        Ok(())
    }
}

#[binrw]
#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

#[bitfield(u8)]
pub struct ImagePacked {
    #[bits(3)]
    pub size_of_local_color_table: u32,

    #[bits(2)]
    pub reserved: usize,

    #[bits(1)]
    pub sort_flag: bool,

    #[bits(1)]
    pub interlace_flag: bool,

    #[bits(1)]
    pub local_color_table_flag: bool,
}

impl BinRead for ImagePacked {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let mut packed = [0u8; 1];
        reader.read_exact(&mut packed)?;
        Ok(ImagePacked::from_bits(packed[0]))
    }
}

pub struct ImagePackedWriteArgs {
    local_color_table_length: usize,
}

impl BinWrite for ImagePacked {
    type Args<'a> = ImagePackedWriteArgs;

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        _endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        writer.write_all(&[self
            .with_local_color_table_flag(args.local_color_table_length != 0)
            .with_size_of_local_color_table(if args.local_color_table_length == 0 {
                0
            } else {
                args.local_color_table_length.ilog2() - 1
            })
            .0])?;
        Ok(())
    }
}

#[binrw]
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub left: u16,
    pub top: u16,
}

impl Position {
    pub const fn new(left: u16, top: u16) -> Self {
        Self { left, top }
    }
}

#[binrw]
#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

#[binrw]
#[derive(Debug)]
pub struct ImagePositioned {
    pub position: Position,
    pub image: Image,
}

#[binrw]
#[derive(Debug)]
pub struct Image {
    pub size: Size,

    #[bw(args_raw = ImagePackedWriteArgs { local_color_table_length: local_color_table.len() })]
    pub packed: ImagePacked,

    /// len is must be 2 or 4 or 8 or 16 or 32 or 64 or 128 or 256
    #[br(count = if packed.local_color_table_flag() {
        2i32.pow(packed.size_of_local_color_table() + 1)
    } else {
        0
    } as usize)]
    #[bw(assert(
        (local_color_table.is_empty() || local_color_table.len().count_ones() == 1)
        && local_color_table.len() <= 256
    ))]
    pub local_color_table: Vec<Color>,

    #[bw(calc(crate::mygif::LZW_CODESIZE))]
    pub _lzw_minimum_code_size: u8,

    #[br(parse_with = chunked_binary_parser)]
    #[bw(write_with = chunked_binary_writer)]
    pub lzw_binary: Vec<u8>,
}

/// WARNING: NON 8 CODESIZE IS NOT SUPPORTED BY FIREFOX
pub const LZW_CODESIZE: u8 = 8;

pub fn do_lzw(data: &[u8]) -> Vec<u8> {
    use lzw::{Encoder, LsbWriter};

    let mut bin = Vec::new();

    let buffer = std::io::Cursor::new(&mut bin);
    let mut encoder = Encoder::new(LsbWriter::new(buffer), LZW_CODESIZE).unwrap();
    encoder.encode_bytes(data).unwrap();
    drop(encoder);

    bin
}

#[binrw::writer(writer)]
fn chunked_binary_writer(binary: &Vec<u8>) -> binrw::BinResult<()> {
    let binary = binary.as_slice();
    let mut cursor = 0usize;

    loop {
        let next_cursor = (cursor + 0xFF).min(binary.len());

        let delta_len = next_cursor - cursor;

        writer.write_all(&[delta_len as u8])?;

        if delta_len == 0 {
            break;
        }

        writer.write_all(&binary[cursor..next_cursor])?;
        cursor = next_cursor;
    }

    Ok(())
}

#[binrw::parser(reader)]
fn chunked_binary_parser() -> binrw::BinResult<Vec<u8>> {
    let mut binary = vec![];
    let mut block_size = [0];

    loop {
        reader.read_exact(&mut block_size)?;
        let block_size = block_size[0];

        if block_size == 0 {
            break;
        }

        let mut chunk = vec![0u8; block_size.into()];
        reader.read_exact(&mut chunk)?;

        binary.extend(chunk);
    }

    Ok(binary)
}

#[binrw::parser(reader, endian)]
fn block_parser() -> binrw::BinResult<Vec<Block>> {
    let mut blocks = Vec::new();

    loop {
        let block = Block::read_options(reader, endian, ())?;
        let is_trailer = matches!(&block, Block::Trailer(_));

        blocks.push(block);

        if is_trailer {
            break;
        }
    }

    Ok(blocks)
}

#[derive(Debug)]
#[repr(u8)]
pub enum DisposalMethod {
    NoDisposalSpecified = 0,
    DoNotDispose = 1,
    RestoreToBackgroundColor = 2,
    RestoreToPrevious = 3,
}

impl DisposalMethod {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0 => Self::NoDisposalSpecified,
            1 => Self::DoNotDispose,
            2 => Self::RestoreToBackgroundColor,
            3 => Self::RestoreToPrevious,
            _ => Self::NoDisposalSpecified,
        }
    }
}

#[bitfield(u8)]
pub struct GraphicControlExtensionPacked {
    #[bits(3)]
    pub _reserved: u8,

    #[bits(3)]
    pub disposal_method: DisposalMethod,

    #[bits(1)]
    pub user_input_flag: bool,

    #[bits(1)]
    pub transpalent_color_flag: bool,
}

#[binrw]
#[derive(Debug)]
pub struct GraphicsControlExtension {
    #[bw(calc(0x04))]
    pub _block_size: u8,

    pub packed: GraphicControlExtensionPacked,

    pub delay_time: u16,

    pub transpalent_color_index: u8,

    #[bw(calc(0x00))]
    pub _terminator: u8,
}

impl BinRead for GraphicControlExtensionPacked {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let mut packed = [0u8; 1];
        reader.read_exact(&mut packed)?;
        Ok(Self::from_bits(packed[0]))
    }
}

impl BinWrite for GraphicControlExtensionPacked {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        writer.write_all(&[self.into_bits()])?;
        Ok(())
    }
}

#[binrw]
#[derive(Debug)]
pub enum Extension {
    #[brw(magic(0xF9u8))]
    GraphicsControlExtension(GraphicsControlExtension),
}

#[binrw]
#[derive(Debug)]
pub enum Block {
    #[brw(magic(0x2Cu8))]
    Image(ImagePositioned),

    #[brw(magic(0x21u8))]
    Extension(Extension),

    #[brw(magic(0x3Bu8))]
    Trailer(()),
}

#[binrw]
#[brw(little, magic = b"GIF")]
#[derive(Debug)]
pub struct Gif {
    pub version: Version,

    pub screen_width: u16,
    pub screen_height: u16,

    #[bw(args_raw = HeaderPackedWriteArgs { global_color_table_length: global_color_table.len() })]
    pub packed: HeaderPacked,

    pub background_color_index: u8,

    /// Most gif parser supports 1:1 only
    pub pixel_aspect_ratio: u8,

    #[br(count = if packed.global_color_table_flag() {
        2i32.pow(packed.size_of_global_color_table() + 1)
    } else {
        0
    } as usize)]
    #[bw(assert(
        (global_color_table.is_empty() || global_color_table.len().count_ones() == 1)
        && global_color_table.len() <= 256
    ))]
    pub global_color_table: Vec<Color>,

    #[br(parse_with = block_parser)]
    pub blocks: Vec<Block>,
}
