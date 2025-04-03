use std::io::Cursor;

use crate::{
    model::Context,
    mygif::{
        Block, Color, Extension, Gif, GraphicControlExtensionPacked, GraphicsControlExtension,
        HeaderPacked, Image, ImagePacked, ImagePositioned, Position, Size, Version,
    },
    Clock, ConnectionCounter,
};

use async_stream::try_stream;
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, HeaderName},
    response::IntoResponse,
};
use binrw::BinWrite;
use bytes::Bytes;
use futures::Stream;
use once_cell::sync::OnceCell;

const GLYPH_COUNT: usize = 13;
const FONT_SIZE: Size = Size::new(6, 10);

static LZW_ENCODED_FONTS: OnceCell<[bytes::Bytes; GLYPH_COUNT]> = OnceCell::new();
static LZW_ENCODED_BG: OnceCell<bytes::Bytes> = OnceCell::new();
static GIF_HEADER: OnceCell<bytes::Bytes> = OnceCell::new();
static GLYPHS: [&[u8; (FONT_SIZE.width * FONT_SIZE.height) as usize]; GLYPH_COUNT] = [
    b"\
_####_\
######\
##__##\
##__##\
##__##\
##__##\
##__##\
##__##\
######\
_####_\
",
    b"\
___##_\
_####_\
_####_\
___##_\
___##_\
___##_\
___##_\
___##_\
___##_\
___##_\
",
    b"\
_####_\
######\
____##\
____##\
_#####\
_####_\
##____\
##____\
######\
######\
",
    b"\
#####_\
######\
____##\
____##\
######\
######\
____##\
____##\
######\
#####_\
",
    b"\
##__##\
##__##\
##__##\
##__##\
######\
######\
____##\
____##\
____##\
____##\
",
    b"\
######\
######\
##____\
##____\
#####_\
_#####\
____##\
____##\
######\
#####_\
",
    b"\
_####_\
######\
##____\
##____\
#####_\
######\
##__##\
##__##\
######\
_####_\
",
    b"\
######\
######\
##__##\
##__##\
____##\
____##\
____##\
____##\
____##\
____##\
",
    b"\
_####_\
######\
##__##\
##__##\
######\
######\
##__##\
##__##\
######\
_####_\
",
    b"\
_####_\
######\
##__##\
##__##\
######\
_#####\
____##\
____##\
######\
_####_\
",
    b"\
______\
______\
______\
______\
######\
######\
______\
______\
______\
______\
",
    b"\
______\
______\
__##__\
__##__\
______\
______\
______\
__##__\
__##__\
______\
",
    b"\
######\
##__##\
#____#\
###__#\
###__#\
##___#\
##__##\
######\
##__##\
######\
",
];

fn to_codepoint(c: u8) -> usize {
    match c {
        b'0' => 0,
        b'1' => 1,
        b'2' => 2,
        b'3' => 3,
        b'4' => 4,
        b'5' => 5,
        b'6' => 6,
        b'7' => 7,
        b'8' => 8,
        b'9' => 9,
        b'-' => 10,
        b':' => 11,
        _ => 12,
    }
}

#[allow(clippy::identity_op)]
#[allow(clippy::erasing_op)]
pub fn encode(ctx: &Context) -> bytes::Bytes {
    let mut buffer = Cursor::new(vec![]);

    let yea_a = ctx.jst.as_bytes()[0];
    let yea_b = ctx.jst.as_bytes()[1];
    let yea_c = ctx.jst.as_bytes()[2];
    let yea_d = ctx.jst.as_bytes()[3];
    let sep_ym = ctx.jst.as_bytes()[4];
    let mon_h = ctx.jst.as_bytes()[5];
    let mon_l = ctx.jst.as_bytes()[6];
    let sep_md = ctx.jst.as_bytes()[7];
    let day_h = ctx.jst.as_bytes()[8];
    let day_l = ctx.jst.as_bytes()[9];
    let _ = ctx.jst.as_bytes()[10];
    let hou_h = ctx.jst.as_bytes()[11];
    let hou_l = ctx.jst.as_bytes()[12];
    let sep_hm = ctx.jst.as_bytes()[13];
    let min_h = ctx.jst.as_bytes()[14];
    let min_l = ctx.jst.as_bytes()[15];
    let sep_ms = ctx.jst.as_bytes()[16];
    let sec_h = ctx.jst.as_bytes()[17];
    let sec_l = ctx.jst.as_bytes()[18];

    let glyphs = &LZW_ENCODED_FONTS.get().unwrap();

    let blocks = [
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(0, 0),
            image: Image {
                size: Size::new(88, 31),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: LZW_ENCODED_BG.get().unwrap().to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 + 7 * 7, 16),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(sec_l)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 + 7 * 6, 16),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(sec_h)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 + 7 * 5, 16),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(sep_ms)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 + 7 * 4, 16),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(min_l)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 + 7 * 3, 16),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(min_h)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 + 7 * 2, 16),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(sep_hm)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 + 7 * 1, 16),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(hou_l)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 + 7 * 0, 16),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(hou_h)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 * 0, 4),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(yea_a)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 * 1, 4),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(yea_b)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 * 2, 4),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(yea_c)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 * 3, 4),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(yea_d)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 * 4, 4),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(sep_ym)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 * 5, 4),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(mon_h)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 * 6, 4),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(mon_l)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 * 7, 4),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(sep_md)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 * 8, 4),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(day_h)].to_vec(),
            },
        }),
        Block::Extension(Extension::GraphicsControlExtension(
            GraphicsControlExtension {
                delay_time: 2,
                transpalent_color_index: 0,
                packed: GraphicControlExtensionPacked::new(),
            },
        )),
        Block::Image(ImagePositioned {
            position: Position::new(9 + 7 * 9, 4),
            image: Image {
                size: Size::new(6, 10),
                packed: ImagePacked::new(),
                local_color_table: vec![],
                lzw_binary: glyphs[to_codepoint(day_l)].to_vec(),
            },
        }),
    ];

    blocks.write_le(&mut buffer).unwrap();

    bytes::Bytes::from(buffer.into_inner())
}

fn stream(
    mut clock: Clock,
    counter: ConnectionCounter,
) -> impl Stream<Item = Result<Bytes, Box<dyn std::error::Error + 'static + Send + Sync>>> {
    try_stream! {
        let _session = counter.acquire();
        yield GIF_HEADER.get().unwrap().clone();
        clock.mark_unchanged();

        loop {
            let _ = clock.changed().await;
            let partial = clock.borrow_and_update().gif.clone();
            yield partial;
        }
    }
}

pub async fn banner_page_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html")],
        include_str!("../assets/banner.html"),
    )
}

pub async fn gif_handler(
    headers: HeaderMap,
    State((clock, counter)): State<(Clock, ConnectionCounter)>,
) -> impl IntoResponse {
    let stream = stream(clock, counter);
    let body = Body::from_stream(stream);

    let is_cloudflare = headers.contains_key("cf-ray");

    let headers = [
        (
            header::CONTENT_TYPE,
            if is_cloudflare {
                "application/grpc"
            } else {
                "image/gif"
            },
        ),
        (
            HeaderName::from_static("x-original-content-type"),
            "image/gif",
        ),
    ];

    (headers, body)
}

fn init_image() {
    let data: [[u8; (FONT_SIZE.width * FONT_SIZE.height) as usize]; GLYPH_COUNT] = GLYPHS.map(|v| {
        v.map(|c| match c {
            b'_' => 0u8,
            b'#' => 1u8,
            _ => 0u8,
        })
    });

    let data: [bytes::Bytes; GLYPH_COUNT] =
        data.map(|raw| bytes::Bytes::from(crate::mygif::do_lzw(&raw)));

    LZW_ENCODED_FONTS.set(data).unwrap();

    let bg = bytes::Bytes::from(crate::mygif::do_lzw(&[0x00; 88 * 31]));
    LZW_ENCODED_BG.set(bg).unwrap();
}

fn init_header() {
    let mut buffer = Cursor::new(vec![]);

    let mygif = Gif {
        version: Version::GIF89a,
        screen_width: 88,
        screen_height: 31,
        packed: HeaderPacked::new()
            .with_global_color_table_flag(true)
            .with_color_resolution(7),
        background_color_index: 0x00,
        pixel_aspect_ratio: 0,
        global_color_table: [
            Color::from_rgb(0x00, 0x00, 0x00),
            Color::from_rgb(0xFF, 0xFF, 0xFF),
        ]
        .into(),
        blocks: [
            Block::Extension(Extension::GraphicsControlExtension(
                GraphicsControlExtension {
                    delay_time: 2,
                    transpalent_color_index: 0,
                    packed: GraphicControlExtensionPacked::new(),
                },
            )),
            Block::Image(ImagePositioned {
                position: Position::new(0, 0),
                image: Image {
                    size: Size::new(88, 31),
                    packed: ImagePacked::new(),
                    local_color_table: vec![],
                    lzw_binary: LZW_ENCODED_BG.get().unwrap().to_vec(),
                },
            }),
        ]
        .into(),
    };

    mygif.write(&mut buffer).unwrap();

    GIF_HEADER
        .set(bytes::Bytes::from(buffer.into_inner()))
        .unwrap();
}

pub fn initialization() {
    init_image();
    init_header();
}
