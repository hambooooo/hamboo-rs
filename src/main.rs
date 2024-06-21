use std::io::Write;

fn main() -> std::io::Result<()> {
    let root_path = std::env::current_dir().unwrap();
    let disk_name = root_path.join("./ui/images/face-picture-hamboo.simg");
    let image_bytes = include_bytes!("../ui/images/face-picture-hamboo.png");
    let header = minipng::decode_png_header(image_bytes).expect("bad PNG");
    let mut buffer = vec![0u8; header.required_bytes()];
    let image = minipng::decode_png(image_bytes, &mut buffer).expect("bad PNG");
    println!("{}×{} image", image.width(), image.height());
    let serializable_image = SerializableImage::new(image.width(), image.height(), image.pixels().to_vec());
    let serialized_image = serializable_image.serialize();
    println!("image path {}", disk_name.clone().to_str().unwrap());
    let mut file = std::fs::File::create(disk_name.clone())?;
    file.write_all(serialized_image.as_slice())?;
    println!("Successfully write bytes to file {:?}", disk_name);
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
