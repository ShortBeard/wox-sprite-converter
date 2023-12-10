//////////////////////////////
///
/// World Of Xeen - Sprite Converter
/// Author: ShortBeard
/// Website: https://saveeditors.com
///
/////////////////////////////
use image::{Rgba, RgbaImage};
use std::fs::File;
use std::io::Read;
use std::{env, io, panic};
mod sprite_convert;

#[derive(Default, Clone)]
struct CellData {
    offset: u16,
    x_offset: u16,
    width: u16,
    y_offset: u16,
    height: u16,
    cell_color_data: Vec<u8>,
}

#[derive(Default, Clone)]
struct Frame {
    cell_data: Vec<CellData>,     //Will either be 1 or 2 in length
    combined_color_data: Vec<u8>, //Store our final color data after cominbing cells (if applicable)
    frame_height: u16,
    frame_width: u16,
    y_offset: u16,
    two_cells: bool,
}

impl Frame {
    fn new() -> Self {
        Frame {
            cell_data: Vec::new(), // Initialize the vector here
            combined_color_data: Vec::new(),
            frame_height: 0,
            frame_width: 0,
            y_offset: 0,
            two_cells: false,
        }
    }

    fn push_cell_offset(&mut self, offset: u16) {
        let mut cell_data = CellData::default();
        cell_data.offset = offset;
        self.cell_data.push(cell_data);
        if self.cell_data.len() == 2 {
            self.two_cells = true;
        }
    }

    //A frames height is always the largest of the 2 cells
    fn set_frame_height(&mut self) {
        //Get whichever cell is our talles (cell height + y_offset determines total height before deciding)
        let tallest_cell = self
            .cell_data
            .iter()
            .max_by_key(|cell| cell.height + cell.y_offset)
            .unwrap();

        //Set the frames height and y offset to the tallest cell found
        self.frame_height = tallest_cell.height;
        self.y_offset = tallest_cell.y_offset;
    }

    //A frames height is always the largest of the 2 cells
    fn set_frame_width(&mut self) {
        self.frame_width = std::cmp::max(
            self.cell_data[0].width,
            self.cell_data.get(1).map_or(0, |cell| cell.width),
        );
    }

    fn combine_color_data(&mut self) {
        //If this frame only uses 1 cell, then just use it's color data as the final image
        if self.two_cells == false {
            self.combined_color_data = self.cell_data[0].cell_color_data.clone();
        }
        //Otherwise combine the two cells
        else {
            let longest_length = std::cmp::max(
                self.cell_data[0].cell_color_data.len(),
                self.cell_data[1].cell_color_data.len(),
            );

            let mut combined_color_data: Vec<u8> = vec![0; longest_length as usize];

            let mut i = 0;
            while i < self.cell_data.len() {
                let mut j = 0;
                while j < self.cell_data[i].cell_color_data.len() {
                    let color: u32 = Self::rgba_to_u32(
                        self.cell_data[i].cell_color_data[j],
                        self.cell_data[i].cell_color_data[j + 1],
                        self.cell_data[i].cell_color_data[j + 2],
                        self.cell_data[i].cell_color_data[j + 3],
                    );
                    if color == 0 {
                        j += 4;
                        continue;
                    } else {
                        combined_color_data[j] = self.cell_data[i].cell_color_data[j];
                        combined_color_data[j + 1] = self.cell_data[i].cell_color_data[j + 1];
                        combined_color_data[j + 2] = self.cell_data[i].cell_color_data[j + 2];
                        combined_color_data[j + 3] = self.cell_data[i].cell_color_data[j + 3];
                    }
                    j += 4;
                }
                i += 1;
            }

            self.combined_color_data = combined_color_data;
        }
    }

    fn rgba_to_u32(r: u8, g: u8, b: u8, a: u8) -> u32 {
        let red = (r as u32) << 24;
        let green = (g as u32) << 16;
        let blue = (b as u32) << 8;
        let alpha = a as u32;
        red | green | blue | alpha
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: .\\xeen-sprite-convert <file_name> <pal_name>");
        eprintln!("Example: .\\xeen-sprite-convert 000.ATT MM4.PAL");
        std::process::exit(1);
    } else {
        let file_name = &args[1];
        let pal_name = &args[2];
        let open_file_result: Result<File, io::Error> = open_file(file_name);
        match open_file_result {
            Ok(file) => {
                let open_pal_result: Result<File, io::Error> = open_file(pal_name);
                match open_pal_result {
                    Ok(pal_file) => begin_image_extraction(file, file_name, pal_file),
                    Err(err) => print!("Error opening PAL file: {}", err),
                }
            }
            Err(err) => print!("Error opening sprite file: {}", err),
        };
    }
}

fn open_file(file_name: &str) -> Result<File, io::Error> {
    let f: File = File::open(file_name)?;
    Ok(f)
}

//Get the bytes from our file and return them as a u8 vector
fn read_bytes(file: &mut File) -> Vec<u8> {
    let mut file_buffer: Vec<u8> = Vec::new();
    let read_result = file.read_to_end(&mut file_buffer);
    match read_result {
        Ok(_) => file_buffer,
        Err(err) => {
            println!("Error while reading file: {} ", err);
            panic!()
        }
    }
}

fn begin_image_extraction(mut sprite_file: File, sprite_file_name: &str, mut pal_file: File) {
    let file_buffer: Vec<u8> = read_bytes(&mut sprite_file);
    let pal_bytes = read_bytes(&mut pal_file);
    let transparent: u32 = 0x00000000; // Transparent RGBA
    let frame_count: u16 = u16::from_le_bytes([file_buffer[0], file_buffer[1]]);

    println!("Frame count: {}", frame_count); //Print how many frames are in this fle

    let mut image_frames: Vec<Frame> = get_cell_offset_info_new(frame_count, &file_buffer);

    //Read the cell headers and get the max height for each frame
    read_cell_headers(&mut image_frames, &file_buffer);
    for frame in &mut image_frames {
        frame.set_frame_height();
        frame.set_frame_width();
    }

    //Convert the cell data to color data
    for frame in image_frames.iter_mut() {
        for cell in frame.cell_data.iter_mut() {
            cell.cell_color_data =
                sprite_convert::cell_to_rgb32(&cell, &pal_bytes, transparent, &file_buffer);
        }
    }

    //Begin creating the images from our cell data
    for (i, frame) in image_frames.iter_mut().enumerate() {
        frame.combine_color_data();
        create_image_from_color_data(
            &frame.combined_color_data,
            frame.frame_width,
            frame.frame_height + frame.y_offset,
            i as u32,
            sprite_file_name,
        );
    }

    println!("Image extraction complete!");
}

fn read_cell_headers(frames: &mut Vec<Frame>, file_buffer: &Vec<u8>) {
    let mut i = 0;

    while i < frames.len() {
        let mut j = 0;
        while j < frames[i].cell_data.len() {
            let current_offset: usize = frames[i].cell_data[j].offset as usize;
            frames[i].cell_data[j].x_offset =
                u16::from_le_bytes([file_buffer[current_offset], file_buffer[current_offset + 1]]);
            frames[i].cell_data[j].width = u16::from_le_bytes([
                file_buffer[current_offset + 2],
                file_buffer[current_offset + 3],
            ]);
            frames[i].cell_data[j].y_offset = u16::from_le_bytes([
                file_buffer[current_offset + 4],
                file_buffer[current_offset + 5],
            ]);
            frames[i].cell_data[j].height = u16::from_le_bytes([
                file_buffer[current_offset + 6],
                file_buffer[current_offset + 7],
            ]);
            println!("Processing: Frame - {}, Cell - {}", i, j);
            j += 1;
        }
        i += 1;
    }
}

fn create_image_from_color_data(
    color_data: &[u8],
    width: u16,
    height: u16,
    counter: u32,
    file_name: &str,
) {
    let mut image = RgbaImage::new(width as u32, height as u32);

    for y in 0..height as u32 {
        for x in 0..width as u32 {
            let index = (y * width as u32 + x) as usize * 4;
            let a = color_data[index + 3];
            let r = color_data[index + 2];
            let g = color_data[index + 1];
            let b = color_data[index + 0];

            image.put_pixel(x, y, Rgba([r, g, b, a]));
        }
    }

    let counter_as_string = counter.to_string();
    let file_name = file_name.to_owned() + "_" + &counter_as_string + ".png";
    println!("Creating Image: {}", file_name);

    //Save the sprite as a PNG
    let save_result = image.save(&file_name);
    if let Err(e) = save_result {
        println!("Error while saving file: {} - {}", &file_name, e);
    }
}

//Grab the cell offset info immediately following the frame count byte
fn get_cell_offset_info_new(frame_count: u16, file_buffer: &Vec<u8>) -> Vec<Frame> {
    let mut image_frames: Vec<Frame> = vec![Frame::new(); frame_count as usize];
    let mut i: usize = 0;

    while i < frame_count as usize {
        let offset_1: u16 =
            u16::from_le_bytes([file_buffer[2 + (i * 4)], file_buffer[3 + (i * 4)]]);
        let offset_2: u16 =
            u16::from_le_bytes([file_buffer[4 + (i * 4)], file_buffer[5 + (i * 4)]]);

        image_frames[i as usize].push_cell_offset(offset_1);

        //2nd offset has potential to be zero, check to make sure it isnt
        if offset_2 != 0 {
            image_frames[i as usize].push_cell_offset(offset_2);
        }

        i += 1;
    }

    image_frames
}
