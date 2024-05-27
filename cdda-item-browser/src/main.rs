use std::fs;
use std::io;
use anyhow::Ok;
use egui::RichText;
use serde_json::Value;
use eframe::egui;
use anyhow::Result;
use std::path::PathBuf;

struct Item { // Representation of a CDDA item
    data: Value,
}


fn get_property(data: &Value, property: &str) -> Option<String> { // Function to get a property from a JSON object
    data.get(property)?.as_str().map(|s| s.to_string())
}

fn get_name(item: &Item) -> Option<String> { // Function to get the name of an item
    // Access the 'name' field and its 'str' subfield
    if let Some(name) = item.data.get("name") { // Check if the 'name' field exists
        if let Some(name_str) = name.get("str") { // Check if the 'str' subfield exists
            if let Some(name_str) = name_str.as_str() { // Check if the 'str' subfield is a string
                return Some(name_str.to_string());
            }
        }
    }
    None // Return None if the name is not found
}

// TODO: Fix UI spacing issues
// TODO: Recursively add directories containing item json files
// TODO: Add a way to find the game directory

fn main() -> Result<()>{
     // Load all the json files in the json directory
    let json_files: Result<Vec<PathBuf>, io::Error> = fs::read_dir("./json")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<PathBuf>, io::Error>>()
        .map_err(|err| err.into()); // Convert the error type

    // Vector to store the Item structs
    let mut items: Vec<Item> = Vec::new();

    // For each file, make an Item struct for each item in the file and put it into the vector
    for file in json_files?.iter().map(|file| file.as_ref()) { // Use the ? operator to propagate the error
        let file_contents = fs::read_to_string::<&std::path::Path>(file)?; // Specify the type annotation for the file variable
        let data: Vec<Value> = serde_json::from_str(&file_contents)?; // Parse the JSON data
        for item_data in data { // Iterate over the items inside the JSON file
            let item = Item { // Create a new Item struct
                data: item_data,
            };
            items.push(item);
        }
    }

    items.sort_by(|a, b| { // Sort the items by name
        let name_a = get_name(&a);
        let name_b = get_name(&b);
    
        match (name_a, name_b) { // Compare the names
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
    
    // Remove all unnamed items
    items.retain(|item| get_name(&item).is_some());
    // Remove all "" named items
    items.retain(|item| get_name(&item).unwrap() != "");

    // Application options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([700.0, 600.0]),
        ..Default::default()
    };
    
    // Application state variables
    let mut selected_item: Option<usize> = None;
    let mut search_text = String::new();

    // Run the application
    let _ = eframe::run_simple_native("Cataclysm: Dark Days Ahead Item Browser", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| { // Central panel
            ui.heading("Cataclysm: Dark Days Ahead Item Browser");
            // Search bar
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut search_text);
            });
            // Box to contain list of items
            ui.separator();
            ui.heading("Items");
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| { // Scroll area
                for (index, item) in items.iter().enumerate() {
                    if let Some(name) = item.data.get("name") { // Check if the 'name' field exists
                        if let Some(name_str) = name.get("str") { // Check if the 'str' subfield exists
                            if let Some(name_str) = name_str.as_str() { // Check if the 'str' subfield is a string
                                // Skip if the name is ""
                                if name_str != "" && name_str.to_lowercase().contains(search_text.to_lowercase().as_str()) {
                                    ui.horizontal(|ui| {
                                        if ui.selectable_value(&mut selected_item, Some(index), name_str).clicked() {
                                            selected_item = Some(index);
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
            });
        });
        // Item properties side panel
        egui::SidePanel::right("side_panel").default_width(300.0).show(ctx, |ui| {
            ui.heading("Information");
            ui.separator();
            if let Some(index) = selected_item {
                // Display the information of the selected item
                ui.heading(format!("{}", get_name(&items[index]).unwrap()));
                // Description
                if let Some(description) = get_property(&items[index].data, "description") {
                    ui.label(RichText::new(description));
                }
                ui.separator();
                // Volume
                if let Some(volume) = get_property(&items[index].data, "volume") {
                    ui.label(format!("Volume: {}", volume));
                }
                // Weight
                if let Some(weight) = get_property(&items[index].data, "weight") {
                    ui.label(format!("Weight: {}", weight));
                }
                // Price
                if let Some(price) = get_property(&items[index].data, "price") {
                    ui.label(format!("Price: {}", price));
                }
                // Material
                if let Some(material) = get_property(&items[index].data, "material") {
                    ui.label(format!("Material: {}", material));
                }
                // Flags
                if let Some(flags) = items[index].data.get("flags") {
                    ui.label("Flags:");
                    for flag in flags.as_array().unwrap() {
                        ui.label(format!("{}", flag.as_str().unwrap()));
                    }
                }
                ui.separator();
                // Display the rest of the properties
                for (key, value) in items[index].data.as_object().unwrap() {
                    if key != "name" && key != "description" && key != "volume" && key != "weight" && key != "price" && key != "material" && key != "flags" {
                        ui.label(format!("{}: {}", key, value));
                    }
                }
            } else {
                ui.label("No item selected");
            }
        });
    });
    Ok(())
}