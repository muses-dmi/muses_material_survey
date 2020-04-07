//! Handle outside world interactions
//! 
//! Copyright: Benedict R. Gaster
//! 
//! 

extern crate uuid;
use uuid::Uuid;

use std::fs::File;

extern crate csv;

use crate::msg;

pub type Contacts = Vec<(u128, f32, f32, f32)>;

pub struct PressData {
    /// radius of circle at start
    pub circle_radius: f32,
    /// radius of ring
    pub ring_radius: f32,
    /// contacts record timestamp, pressure, x, and y
    pub contacts: Contacts,
    /// did the user complete the press test for given params
    pub success: bool,
}

///
pub struct World {
    pub id: Uuid,
    /// likert CSV file
    pub csv: csv::Writer<File>,
}

impl World {
    pub fn new(id: Uuid, likert_file: File) -> Self {

        // handle Likert CSV files, including writing column headings
        let mut csv = csv::Writer::from_writer(likert_file);
        csv.write_record(&["ID", "Category", "Gesture", "Material", "Feeling", "Answer"]);
        csv.flush().unwrap();
        
        World {
            id: id,
            csv: csv,
        }
    }

    /// create an ID label to be written at front of each new entry in CSV
    pub fn create_id(&self) -> String {
        self.id.to_hyphenated().to_string()
    }

    /// write likert data to CSV
    pub fn writeLikert(&mut self, gesture: &str, material: &str, likert: msg::Likert) {
        let categories = ["Strongly Disagree", "Disagree", "Netural", "Agree", "Strongly Agree"];

        self.csv.write_record(&[
            &self.create_id(), 
            categories[(likert.value-1) as usize],
            gesture, material, 
            &likert.name, 
            &likert.value.to_string()]).unwrap();
        self.csv.flush().unwrap();
    }

    /// write respose data out to CSV
    pub fn writeResponse(&mut self, name: String, materials: Vec<String>) {
        let mut out = vec![self.create_id(), name];
        out.extend(materials);
        self.csv.write_record(out);
        self.csv.flush().unwrap();
    }

    /// write gesture data out to CSV
    pub fn writeGesture(
        &mut self,
        name: String,
        material: u32,
        circle_ring_radius: Vec<(f32, f32)>,
        contacts: Vec<Contacts>) {
        
        for i in 0..circle_ring_radius.len() {
            // id=id, press, material, circleRadius, ringRadius, (timestamp, pressure, x, y), ...
            let mut out = vec![
                self.create_id(), 
                name.clone(), 
                material.to_string(),
                circle_ring_radius[i].0.to_string(),
                circle_ring_radius[i].1.to_string()];
            
            for contact in &contacts[i] {
                out.push(format!(
                    "({} - {} - {} - {})", 
                    contact.0.to_string(), 
                    contact.1.to_string(), 
                    contact.2.to_string(),
                    contact.3.to_string()));
            }

            self.csv.write_record(out);
            self.csv.flush().unwrap();
        }
    }

    /// In some cases, when in a loop, for example, we don't want to flush until end of slide.
    pub fn flush_CSV(&mut self) {
        self.csv.flush().unwrap();
    }
}