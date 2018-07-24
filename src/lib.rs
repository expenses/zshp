//only the basic type. measure and z parts are not implement.
extern crate byteorder;
use byteorder::{ByteOrder, BigEndian, LittleEndian};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::default::Default;

#[derive(Debug)]
pub struct ShpFile {
    pub header: ShpHeader,
    pub records: Vec<ShpRecord>,
}

#[derive(Debug)]
pub struct ShpHeader {
    // length is the size of the whole file measured with 16bit, include the 50 word in header file.
    pub length: i32,
    pub version: i32,
    pub shape_type: ShapeType,
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
    pub zmax: Option<f64>,
    pub zmin: Option<f64>,
    pub mmax: Option<f64>,
    pub mmin: Option<f64>,
}
impl ShpHeader {
    pub fn parse(header_buffer: [u8; 100]) -> ShpHeader {
        let mut shp_header: ShpHeader = Default::default();
        let file_code = BigEndian::read_i32(&header_buffer[0..4]);
        if file_code != 9994 {
            panic!("File code error: file code do not equal 9994.");
        }
        shp_header.length = BigEndian::read_i32(&header_buffer[24..28]);
        let version = LittleEndian::read_i32(&header_buffer[28..32]);
        shp_header.version = version;
        if version != 1000 {
            panic!("Version error: shapefile version do not equal 1000.");
        }
        if let Some(shape_type) = ShapeType::from_i32(LittleEndian::read_i32(&header_buffer[32..36])) {
            shp_header.shape_type = shape_type;
        } else {
            panic!("Shape type error.");
        }
        shp_header.xmin = LittleEndian::read_f64(&header_buffer[36..44]);
        shp_header.ymin = LittleEndian::read_f64(&header_buffer[44..52]);
        shp_header.xmax = LittleEndian::read_f64(&header_buffer[52..60]);
        shp_header.ymax = LittleEndian::read_f64(&header_buffer[60..68]);
        let zmin = LittleEndian::read_f64(&header_buffer[68..76]);
        let zmax = LittleEndian::read_f64(&header_buffer[76..84]);
        let mmin = LittleEndian::read_f64(&header_buffer[84..92]);
        let mmax = LittleEndian::read_f64(&header_buffer[92..100]);
        if zmin == 0.0 && zmax == 0.0 {
            shp_header.zmin = None;
            shp_header.zmax = None;
        } else {
            shp_header.zmin = Some(zmin);
            shp_header.zmax = Some(zmax);
        }
        if mmin == 0.0 && mmax == 0.0 {
            shp_header.mmin = None;
            shp_header.mmax = None;
        } else {
            shp_header.mmin = Some(mmin);
            shp_header.mmax = Some(mmax);
        }
        shp_header
    }
}
impl Default for ShpHeader {
    fn default() -> ShpHeader {
        ShpHeader {
            length: 0,
            version: 1000,
            shape_type: ShapeType::Null,
            xmin: 0.0,
            xmax: 0.0,
            ymin: 0.0,
            ymax: 0.0,
            zmax: None,
            zmin: None,
            mmax: None,
            mmin: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeType {
    Null = 0,
    Point = 1,
    Polyline = 3,
    Polygon = 5,
    MultiPoint = 8,
    PointZ = 11,
    PolylineZ = 13,
    PolygonZ = 15,
    MultiPointZ = 18,
    PointM = 21,
    PolylineM = 23,
    PolygonM = 25,
    MultiPointM = 28,
    MultiPatch = 31,
}

impl ShapeType {
    pub fn from_i32(number: i32) -> Option<ShapeType> {
        match number {
            0  => Some(ShapeType::Null),
            1  => Some(ShapeType::Point),
            3  => Some(ShapeType::Polyline),
            5  => Some(ShapeType::Polygon),
            8  => Some(ShapeType::MultiPoint),
            11 => Some(ShapeType::PointZ),
            13 => Some(ShapeType::PolylineZ),
            15 => Some(ShapeType::PolygonZ),
            18 => Some(ShapeType::MultiPointZ),
            21 => Some(ShapeType::PointM),
            23 => Some(ShapeType::PolylineM),
            25 => Some(ShapeType::PolygonM),
            28 => Some(ShapeType::MultiPointM),
            31 => Some(ShapeType::MultiPatch),
            _  => None
        }
    }
}
#[derive(Debug)]
pub struct ShpRecord {
    pub header: ShpRecordHeader,
    pub content: ShpRecordContent,
}
#[derive(Debug)]
pub struct ShpRecordHeader {
    pub number: i32,
    //length means the length of record's content, not include the length of header file.
    pub length: i32,
}
impl ShpRecordHeader {
    pub fn parse(record_header_buffer: &[u8]) -> ShpRecordHeader {
        let record_number  = BigEndian::read_i32(&record_header_buffer[0..4]);
        let content_length = BigEndian::read_i32(&record_header_buffer[4..8]);
        ShpRecordHeader {
            number: record_number,
            length: content_length,
        }
    }
}
#[derive(Debug)]
pub enum ShpRecordContent {
    Null,
    Point(PointShape),
    MultiPoint(MultiPointShape),
    PolyLine(PolyLineShape),
    Polygon(PolygonShape),
}
impl ShpRecordContent {
    pub fn parse(record_content: &[u8]) -> ShpRecordContent {
        let shape_type = ShapeType::from_i32(LittleEndian::read_i32(&record_content[0..4])).unwrap();
        let shp_record_content = match shape_type {
            ShapeType::Null => ShpRecordContent::Null,
            ShapeType::Point => ShpRecordContent::Point(PointShape::parse(&record_content[4..])),
            ShapeType::MultiPoint => ShpRecordContent::MultiPoint(MultiPointShape::parse(&record_content[4..])),
            ShapeType::Polyline => ShpRecordContent::PolyLine(PolyLineShape::parse(&record_content[4..])),
            ShapeType::Polygon => ShpRecordContent::Polygon(PolygonShape::parse(&record_content[4..])),
            _ => panic!("Not Implement!"),
        };
        shp_record_content
    }
}
#[derive(Debug)]
pub struct PointShape {
    pub x: f64,
    pub y: f64,
}
impl PointShape {
    pub fn parse(content: &[u8]) -> PointShape {
        PointShape {
            x: LittleEndian::read_f64(&content[0..8]),
            y: LittleEndian::read_f64(&content[8..16]),
        }
    }
}
#[derive(Debug)]
pub struct MultiPointShape {
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
    pub num_points: i32,
    pub points: Vec<PointShape>,
}
impl MultiPointShape {
    pub fn parse(content: &[u8]) -> MultiPointShape {
        let xmin = LittleEndian::read_f64(&content[0..8]);
        let ymin = LittleEndian::read_f64(&content[8..16]);
        let xmax = LittleEndian::read_f64(&content[16..24]);
        let ymax = LittleEndian::read_f64(&content[24..32]);
        let num_points = LittleEndian::read_i32(&content[32..36]);
        let mut points: Vec<PointShape> = Vec::new();
        let content = &content[36..];
        for i in 0..num_points {
            points.push(PointShape::parse(&content[16*i as usize..16+16*i as usize]))
        }
        MultiPointShape {
            xmin: xmin,
            ymin: ymin,
            xmax: xmax,
            ymax: ymax,
            num_points: num_points,
            points: points,
        }
    }
}
#[derive(Debug)]
pub struct PolyLineShape {
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
    pub num_parts: i32,
    pub num_points: i32,
    pub parts: Vec<i32>,
    pub points: Vec<PointShape>,
}
impl PolyLineShape {
    pub fn parse(content: &[u8]) -> PolyLineShape {
        let xmin = LittleEndian::read_f64(&content[0..8]);
        let ymin = LittleEndian::read_f64(&content[8..16]);
        let xmax = LittleEndian::read_f64(&content[16..24]);
        let ymax = LittleEndian::read_f64(&content[24..32]);
        let num_parts = LittleEndian::read_i32(&content[32..36]);
        let num_points = LittleEndian::read_i32(&content[36..40]);
        let mut parts: Vec<i32> = Vec::new();
        let mut points: Vec<PointShape> = Vec::new();
        let content = &content[40..];
        for i in 0..num_parts {
            parts.push(LittleEndian::read_i32(&content[4*i as usize..4+4*i as usize]));
        }
        let content = &content[4*num_parts as usize ..];
        for j in 0..num_points {
            points.push(PointShape::parse(&content[16*j as usize .. 16+16*j as usize]));
        }
        PolyLineShape {
            xmin: xmin,
            xmax: xmax,
            ymin: ymin,
            ymax: ymax,
            num_parts: num_parts,
            num_points: num_points,
            parts: parts,
            points: points,
        }
    }
}
#[derive(Debug)]
pub struct PolygonShape {
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
    pub num_parts: i32,
    pub num_points: i32,
    //每一个part的首位两点必须相同  
    pub parts: Vec<i32>,
    pub points: Vec<PointShape>,
}
impl PolygonShape {
    pub fn parse(content: &[u8]) -> PolygonShape {
        let xmin = LittleEndian::read_f64(&content[0..8]);
        let ymin = LittleEndian::read_f64(&content[8..16]);
        let xmax = LittleEndian::read_f64(&content[16..24]);
        let ymax = LittleEndian::read_f64(&content[24..32]);
        let num_parts = LittleEndian::read_i32(&content[32..36]);
        let num_points = LittleEndian::read_i32(&content[36..40]);
        let mut parts: Vec<i32> = Vec::new();
        let mut points: Vec<PointShape> = Vec::new();
        let content = &content[40..];
        for i in 0..num_parts {
            parts.push(LittleEndian::read_i32(&content[4*i as usize..4+4*i as usize]));
        }
        let content = &content[4*num_parts as usize ..];
        for j in 0..num_points {
            points.push(PointShape::parse(&content[16*j as usize .. 16+16*j as usize]));
        }
        PolygonShape {
            xmin: xmin,
            xmax: xmax,
            ymin: ymin,
            ymax: ymax,
            num_parts: num_parts,
            num_points: num_points,
            parts: parts,
            points: points,
        }
    }
}

pub struct ShpReader {
    pub shp_header_buffer: [u8; 100],
    pub record_buffer: Vec<u8>,
}

impl ShpReader {
    pub fn new() -> ShpReader {
        ShpReader {
            shp_header_buffer: [0; 100],
            record_buffer: Vec::new(),
        }
    }
    pub fn open<P: AsRef<Path>> (file_path: P) -> Result<ShpReader, io::Error> {
        let mut shp_reader = ShpReader::new();
        let shp_file_path = Path::new(file_path.as_ref()).with_extension("shp");
        let mut f = File::open(shp_file_path)?;
        f.read(&mut shp_reader.shp_header_buffer)?;           
        f.read_to_end(&mut shp_reader.record_buffer)?;
        Ok(shp_reader)
    }
    pub fn read(&self) -> ShpFile {
        let header = ShpHeader::parse(self.shp_header_buffer);
        let mut shp_file = ShpFile { header: header, records: Vec::new()};
        let mut record_buffer = &self.record_buffer[..];
        while !record_buffer.is_empty() {
            let record_header = ShpRecordHeader::parse(&record_buffer[0..8]);
            let record_end_offset: usize = (record_header.length * 2 + 8) as usize;
            let record_content = ShpRecordContent::parse(&record_buffer[8..record_end_offset]);
            let shp_record = ShpRecord {
                header: record_header,
                content: record_content,
            };
            shp_file.records.push(shp_record);
            record_buffer = &record_buffer[record_end_offset..];
        }    
        shp_file  
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use ShapeType;
        assert!(ShapeType::PointZ as u32 == 11);
    }
}


