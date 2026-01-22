use core::{fmt, time::Duration};
use std::vec;

use pixels::Pixels;
use tinyase::parser::{HeaderReader, chunk::{self, CelData}, frame};
use tiny_skia::{self, IntSize, PixmapMut, PixmapPaint, Transform};



pub struct ASEDrawing<'a> {
    pub reader: HeaderReader<'a>
}


impl<'a> ASEDrawing<'a> {
    pub fn draw(&self, pixel: &mut Pixels, elapsed: Duration) {
        let buf_size = (pixel.texture().size().width, pixel.texture().size().height);

        let mut frame_pixmap = PixmapMut::from_bytes(pixel.frame_mut(), buf_size.0, buf_size.1).unwrap();

        frame_pixmap.fill(tiny_skia::Color::from_rgba8(0x8b, 0x8b, 0x8b, 0xff));

        // --- ANIMATION LOGIC ---
        // Get elapsed time in seconds as a float
        let elapsed = elapsed.as_secs_f32();

        let mut frames = self.reader.frames();
        let frame_1 = frames.next().unwrap();
        let mut chunks = frame_1.chunks();
        for chunk in chunks {
            match chunk {
                chunk::ASEChunk::Cel(c) => {
                    let cd = c.get();
                    match cd {
                        CelData::Raw(raw) => {
                            let ch = c.cel_header;
                            
                            let width = raw.header.width as usize;
                            let height = raw.header.height as usize;
                            let src_ptr = raw.ptr;
                            // for each indexed color of raw.ptr we convert to rgba
                            // create target pixmap array of width and height, 4bpp
                            let mut img_buf = vec![0u8; width * height * 4];

                            for (i, target) in img_buf.chunks_exact_mut(4).enumerate() {
                                match src_ptr[i] {
                                    0 => target.copy_from_slice(&[0,0,0,0]),
                                    1 => target.copy_from_slice(&[22,18,54,255]),
                                    2 => target.copy_from_slice(&[255,255,255,255]),
                                    _ => target.copy_from_slice(&[255,255,255,255]),
                                };
                            }
                            // println!("cel: {:?}", raw.header);


                            let img_pixmap = tiny_skia::Pixmap::from_vec(img_buf, IntSize::from_wh(width as _, height as _).unwrap()).unwrap();
                            // let img_pixmap = tiny_skia::PixmapMut::from_bytes(&mut img_buf, width as _ , height as _ ).unwrap();

                            frame_pixmap.draw_pixmap(ch.point_x as _, ch.point_y as _, img_pixmap.as_ref(), &PixmapPaint::default(), Transform::default(), None);

                            

                        },
                        _ => {}
                    }

                },
                _ => {}
            }
        }



        // // Create a shifting value based on time (cycles every ~2.5 seconds)
        // let time_shift = (elapsed * 100.0) as u32;

        // for (i, px) in frame.chunks_exact_mut(4).enumerate() {
        //     let x = (i % width as usize) as u32;
        //     let y = (i / width as usize) as u32;

        //     // Use time_shift to animate the colors
        //     px[0] = ((x + time_shift) % 255) as u8; // Animated Red
        //     px[1] = ((y + time_shift) % 255) as u8; // Animated Green
        //     px[2] = 160;                            // Static Blue
        //     px[3] = 255;                            // Alpha
        // }

    }

    pub fn size(&self) -> (u32, u32) {
        let header = self.reader.header();
        (header.width as u32, header.height as u32)
    }
}