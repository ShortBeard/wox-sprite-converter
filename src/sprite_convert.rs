use crate::CellData;

pub fn cell_to_rgb32(
    cell_data: &CellData, palette_data: &[u8], transparent: u32, file_buffer: &Vec<u8>,
) -> Vec<u8> {
    let (x_offset, width, y_offset, height) = (
        cell_data.x_offset as usize,
        cell_data.width as usize,
        cell_data.y_offset as usize,
        cell_data.height as usize,
    );

    let total_bytes = (cell_data.x_offset as u32 + cell_data.width as u32)
        * (cell_data.y_offset as u32 + cell_data.height as u32)
        * 4;

    let mut color_data: Vec<u8> = vec![0; total_bytes as usize];

    // Split the RGBA color into its constituent bytes
    let rgba_bytes = transparent.to_be_bytes(); // Big-endian byte order

    // Fill the vector with transparency (with alpha channel, not a placeholder color that acts as such) for PNG format
    for _ in 0..(total_bytes as usize / 4) {
        color_data.extend_from_slice(&rgba_bytes);
    }

    let mut dp: usize = (cell_data.offset + 8) as usize;
    let mut y_pos = y_offset;
    while y_pos < height + y_offset {
        let mut byte_count = 0;
        let line_length = file_buffer[dp] as usize;
        dp += 1;

        if line_length > 0 {
            let mut x_pos = file_buffer[dp] as usize + x_offset;
            dp += 1;
            byte_count += 1;

            while byte_count < line_length {
                let opcode = file_buffer[dp];
                dp += 1;
                byte_count += 1;

                let len = (opcode & 0x1F) as usize;
                let cmd = (opcode & 0xE0) >> 5;

                match cmd {
                    0 | 1 => {
                        for _ in 0..opcode + 1 {
                            let color_index = file_buffer[dp] as usize;
                            dp += 1;
                            byte_count += 1;
                            put_pixel(
                                &mut color_data,
                                x_pos,
                                y_pos,
                                width,
                                color_index,
                                palette_data,
                            );
                            x_pos += 1;
                        }
                    }
                    2 => {
                        let opr1 = file_buffer[dp] as usize;
                        dp += 1;
                        byte_count += 1;
                        for _ in 0..(len + 3) {
                            put_pixel(&mut color_data, x_pos, y_pos, width, opr1, palette_data);
                            x_pos += 1;
                        }
                    }
                    3 => {
                        let opr1 =
                            u16::from_le_bytes([file_buffer[dp], file_buffer[dp + 1]]) as usize;
                        dp += 2;
                        byte_count += 2;

                        let mut i: usize = 0;
                        while i < len + 4 {
                            let array_index: usize = dp - opr1 + i;
                            //let current_offset = dp + cell_data.offset as usize + 8;
                            //let negative_offset = current_offset - array_index;
                            let opr2: usize = file_buffer[array_index as usize] as usize;
                            put_pixel(&mut color_data, x_pos, y_pos, width, opr2, &palette_data);
                            i += 1;
                            x_pos += 1;
                        }
                    }
                    4 => {
                        let opr1 = file_buffer[dp] as usize;
                        let opr2 = file_buffer[dp + 1] as usize;
                        dp += 2;
                        byte_count += 2;
                        for _ in 0..(len + 2) {
                            put_pixel(&mut color_data, x_pos, y_pos, width, opr1, palette_data);
                            put_pixel(&mut color_data, x_pos + 1, y_pos, width, opr2, palette_data);
                            x_pos += 2;
                        }
                    }
                    5 => {
                        x_pos += len + 1;
                    }
                    6 | 7 => {
                        let pattern_steps = [0, 1, 1, 1, 2, 2, 3, 3, 0, -1, -1, -1, -2, -2, -3, -3];
                        let len = opcode & 0x07;
                        let cmd = (opcode >> 2) & 0x0E;
                        let mut value: isize = file_buffer[dp] as isize;
                        dp += 1;
                        byte_count += 1;
                        let mut i = 0;
                        while i < len + 3 {
                            //for i in 0..len + 3 {
                            put_pixel(
                                &mut color_data,
                                x_pos,
                                y_pos,
                                width,
                                value as usize,
                                palette_data,
                            );
                            x_pos += 1;
                            value += pattern_steps[(cmd + (i % 2)) as usize];
                            i += 1;
                        }
                    }
                    _ => {}
                }
            }
        } else {
            y_pos += file_buffer[dp] as usize;
            dp += 1;
        }

        y_pos += 1;
    }

    color_data
}

fn put_pixel(
    color_data: &mut Vec<u8>, x: usize, y: usize, width: usize, color_index: usize,
    palette_data: &[u8],
) {
    color_data[(y * width + x) * 4] = palette_data[color_index * 3 + 2] << 2;
    color_data[(y * width + x) * 4 + 1] = palette_data[color_index * 3 + 1] << 2;
    color_data[(y * width + x) * 4 + 2] = palette_data[color_index * 3] << 2;
    color_data[(y * width + x) * 4 + 3] = 0xFF;
}
