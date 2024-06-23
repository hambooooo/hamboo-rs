use std::error::Error;
use std::io::{Cursor, Write};
use image::io::Reader as ImageReader;

fn main() -> Result<(), Box<dyn Error>> {
    let root_dir = std::env::current_dir().unwrap();
    let images_dir = root_dir.join("./ui/images");

    let image_names = [
        "face-pointer-hour",
        "face-pointer-minute",
        "face-pointer-second",
        "app-calculate",
    ];

    let mut index = 0;
    for image_name in image_names {
        index += 1;
        let raw_image_path = images_dir.join(format!("./{}.png", image_name.clone()));
        let pxs_path = images_dir.join(format!("./{:04}-{}.pxs", index, image_name.clone()));

        let image = ImageReader::open(raw_image_path).unwrap().decode().unwrap();

        println!("{}×{} image", image.width(), image.height());
        let serializable_image = SerializableImage::new(image.width(), image.height(), image.as_bytes().to_vec());
        let serialized_image = serializable_image.serialize();
        println!("image path {}", pxs_path.clone().to_str().unwrap());

        let mut file = std::fs::File::create(pxs_path.clone()).unwrap();
        file.write_all(serialized_image.as_slice()).unwrap();
        println!("Successfully write bytes to file {:?}", pxs_path);
    }

    Ok(())
}


pub struct SerializableImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl SerializableImage {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let width_bytes = self.width.to_be_bytes();
        let height_bytes = self.height.to_be_bytes();

        let mut serialized_data = Vec::new();
        serialized_data.extend_from_slice(&width_bytes);
        serialized_data.extend_from_slice(&height_bytes);
        serialized_data.extend_from_slice(self.data.as_slice());

        serialized_data
    }
}
